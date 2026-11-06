// Private, length-prefixed JSON protocol over stdin/stdout. stdout is reserved
// for protocol bytes; all engine/OpenCV/SeetaFace diagnostics go to stderr.
#include <arpa/inet.h>
#include <unistd.h>
#include <array>
#include <cmath>
#include <filesystem>
#include <future>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <opencv2/imgcodecs.hpp>
#include <opencv2/imgproc.hpp>
#include "cvedix/nodes/common/cvedix_node.h"
#include "cvedix/third_party/nlohmann/json.hpp"
#include "cvedix/third_party/cpp_base64/base64.h"
#ifdef CVEDIX_WITH_FACE
#include "cvedix/nodes/infers/cvedix_face_recognizer_node.h"
#include <seeta/AgePredictor.h>
#endif

using json = nlohmann::json;
using Node = cvedix_nodes::cvedix_node;
using Frame = cvedix_objects::cvedix_frame_meta;
constexpr size_t MAX_REQUEST = 8 * 1024 * 1024;
constexpr size_t MAX_RESPONSE = 2 * 1024 * 1024;

static bool read_exact(void* target, size_t count) {
    auto* bytes = static_cast<char*>(target);
    while (count) {
        auto n = ::read(STDIN_FILENO, bytes, count);
        if (n < 0 && errno == EINTR) continue;
        if (n <= 0) return false;
        bytes += n; count -= static_cast<size_t>(n);
    }
    return true;
}
static bool write_exact(int fd, const void* source, size_t count) {
    auto* bytes = static_cast<const char*>(source);
    while (count) {
        auto n = ::write(fd, bytes, count);
        if (n < 0 && errno == EINTR) continue;
        if (n <= 0) return false;
        bytes += n; count -= static_cast<size_t>(n);
    }
    return true;
}

static cv::Mat decode_image(const json& args) {
    std::string text = args.at("image_base64").get<std::string>();
    if (text.starts_with("data:")) {
        auto comma = text.find(',');
        auto prefix = text.substr(0, comma);
        if (comma == std::string::npos || (prefix != "data:image/jpeg;base64"
            && prefix != "data:image/png;base64" && prefix != "data:image/webp;base64"))
            throw std::runtime_error("Only JPEG, PNG and WebP data URIs are supported");
        text.erase(0, comma + 1);
    }
    if (text.empty() || text.size() > 6 * 1024 * 1024 || text.size() % 4 != 0)
        throw std::runtime_error("Invalid image base64 length (maximum 6 MiB)");
    auto padding = text.find('=');
    size_t data_end = padding == std::string::npos ? text.size() : padding;
    if (padding != std::string::npos && (text.size() - padding > 2
        || text.find_first_not_of('=', padding) != std::string::npos))
        throw std::runtime_error("Invalid base64 padding");
    for (size_t i = 0; i < data_end; ++i) {
        char c = text[i];
        if (!(c >= 'A' && c <= 'Z') && !(c >= 'a' && c <= 'z')
            && !(c >= '0' && c <= '9') && c != '+' && c != '/')
            throw std::runtime_error("Invalid base64 character");
    }
    const auto bytes = base64_decode(text);
    // Reject non-image formats before asking OpenCV to decode them.
    bool png = bytes.size() >= 8 && bytes.compare(0, 8, "\x89PNG\r\n\x1a\n", 8) == 0;
    bool jpeg = bytes.size() >= 3 && static_cast<unsigned char>(bytes[0]) == 0xff
        && static_cast<unsigned char>(bytes[1]) == 0xd8 && static_cast<unsigned char>(bytes[2]) == 0xff;
    bool webp = bytes.size() >= 12 && bytes.compare(0, 4, "RIFF") == 0 && bytes.compare(8, 4, "WEBP") == 0;
    if (!png && !jpeg && !webp) throw std::runtime_error("Unsupported encoded image");
    std::vector<unsigned char> encoded(bytes.begin(), bytes.end());
    auto image = cv::imdecode(encoded, cv::IMREAD_COLOR);
    if (image.empty() || image.total() > 16777216 || image.cols > 8192 || image.rows > 8192)
        throw std::runtime_error("Invalid image or decoded dimensions exceed limits");
    return image.isContinuous() ? image : image.clone();
}

static json options_for(const json& args) {
    const auto& options = args.value("options", json::object());
    if (options.is_null()) return json::object();
    if (!options.is_object()) throw std::runtime_error("options must be an object");
    auto backend = options.value("backend", std::string("cpu"));
    if (backend != "cpu" && backend != "auto")
        throw std::runtime_error("Requested backend is unavailable; this worker supports CPU only");
    return options;
}

class Ingress : public Node {
public:
    Ingress() : Node("heramind_input") { max_in_queue_size = 1; initialized(); }
    ~Ingress() override { deinitialized(); }
};

class ImageProcessor : public Node {
protected:
    std::shared_ptr<cvedix_objects::cvedix_meta> handle_frame_meta(std::shared_ptr<Frame> frame) override {
        try {
            auto request = json::parse(frame->description);
            auto options = request.at("options");
            json result = {{"backend", "cpu"}, {"width", frame->frame.cols},
                {"height", frame->frame.rows}, {"channels", 3}, {"pixel_format", "bgr8"},
                {"engine_frame_id", frame->frame_index}};
            if (request.at("task") == "image.preprocess") {
                int width = options.value("width", frame->frame.cols);
                int height = options.value("height", frame->frame.rows);
                if (width < 1 || height < 1 || width > 2048 || height > 2048)
                    throw std::runtime_error("Output dimensions must be 1–2048");
                cv::Mat output;
                cv::resize(frame->frame, output, {width, height}, 0, 0, cv::INTER_AREA);
                auto format = options.value("format", std::string("bgr"));
                if (format == "gray") cv::cvtColor(output, output, cv::COLOR_BGR2GRAY);
                else if (format != "bgr") throw std::runtime_error("format must be bgr or gray");
                std::vector<unsigned char> jpeg;
                cv::imencode(".jpg", output, jpeg, {cv::IMWRITE_JPEG_QUALITY, 85});
                if (jpeg.size() > 1024 * 1024) throw std::runtime_error("Processed image exceeds 1 MiB");
                result["image_base64"] = "data:image/jpeg;base64," + base64_encode(jpeg.data(), jpeg.size());
                result["output_width"] = width;
                result["output_height"] = height;
                result["output_channels"] = output.channels();
            }
            auto mean = cv::mean(frame->frame);
            result["mean_bgr"] = {mean[0], mean[1], mean[2]};
            frame->description = json({{"data", result}}).dump();
        } catch (const std::exception& e) {
            frame->description = json({{"error", e.what()}}).dump();
        }
        return frame;
    }
public:
    ImageProcessor() : Node("heramind_image_processor") { max_in_queue_size = 1; initialized(); }
    ~ImageProcessor() override { deinitialized(); }
};

class Sink : public Node {
    std::mutex completion_mutex;
    std::shared_ptr<std::promise<std::shared_ptr<Frame>>> completion;
    std::shared_ptr<cvedix_objects::cvedix_meta> handle_frame_meta(std::shared_ptr<Frame> frame) override {
        std::shared_ptr<std::promise<std::shared_ptr<Frame>>> pending;
        { std::lock_guard lock(completion_mutex); pending = std::move(completion); }
        if (pending) pending->set_value(frame);
        return nullptr;
    }
public:
    void expect(std::shared_ptr<std::promise<std::shared_ptr<Frame>>> value) {
        std::lock_guard lock(completion_mutex); completion = std::move(value);
    }
    Sink() : Node("heramind_result") { max_in_queue_size = 1; initialized(); }
    ~Sink() override { deinitialized(); }
};

class Pipeline {
    std::shared_ptr<Ingress> input;
    std::shared_ptr<Node> processor;
    std::shared_ptr<Sink> sink;
    int frame_id = 0;
public:
    explicit Pipeline(std::shared_ptr<Node> node) : input(std::make_shared<Ingress>()), processor(std::move(node)), sink(std::make_shared<Sink>()) {
        processor->attach_to({input});
        sink->attach_to({processor});
    }
    ~Pipeline() {
        input->detach_recursively();
        sink.reset(); processor.reset(); input.reset();
    }
    std::shared_ptr<Frame> run(cv::Mat image, const json& description) {
        auto promise = std::make_shared<std::promise<std::shared_ptr<Frame>>>();
        auto future = promise->get_future();
        sink->expect(promise);
        if (frame_id == std::numeric_limits<int>::max()) frame_id = 0;
        auto frame = std::make_shared<Frame>(image, ++frame_id, 0, image.cols, image.rows, 1);
        frame->description = description.dump();
        input->meta_flow(frame);
        // The bridge's outer timeout kills the child if processing stalls.
        return future.get();
    }
};

class Engine {
    Pipeline images{std::make_shared<ImageProcessor>()};
#ifdef CVEDIX_WITH_FACE
    std::shared_ptr<cvedix_nodes::cvedix_face_recognizer_node> face_node;
    std::unique_ptr<Pipeline> faces;
    std::unique_ptr<seeta::AgePredictor> age_predictor;
    const std::string database = "state/seetaface/faces";
    void ensure_faces() {
        if (faces) return;
        const std::string model_dir = "models/seetaface6";
        for (const auto* name : {"face_detector.csta", "mask_detector.csta", "face_landmarker_pts5.csta",
             "face_recognizer.csta", "face_landmarker_mask_pts5.csta", "face_recognizer_mask.csta"})
            if (!std::filesystem::is_regular_file(model_dir + "/" + name))
                throw std::runtime_error(std::string("Missing SeetaFace model: ") + name);
        std::filesystem::create_directories("state/seetaface");
        face_node = std::make_shared<cvedix_nodes::cvedix_face_recognizer_node>("heramind_seetaface",
            model_dir, database, 0.70f, 40, false, false, 0, cvedix_nodes::FaceRecognizerMode::SYNC);
        if (!face_node->initialize()) { face_node.reset(); throw std::runtime_error("Cannot initialize SeetaFace models"); }
        faces = std::make_unique<Pipeline>(face_node);
    }
    void ensure_age() {
        if (age_predictor) return;
        const std::string path = "models/seetaface6/age_predictor.csta";
        if (!std::filesystem::is_regular_file(path))
            throw std::runtime_error("Missing SeetaFace age model: age_predictor.csta");
        seeta::ModelSetting setting(path, seeta::ModelSetting::CPU, 0);
        auto predictor = std::make_unique<seeta::AgePredictor>(setting);
        predictor->set(seeta::AgePredictor::PROPERTY_NUMBER_THREADS, 2);
        age_predictor = std::move(predictor);
    }
#endif
public:
    json execute(const std::string& task, const json& args) {
        auto options = options_for(args);
        if (task == "health") {
            json tasks = {"image.inspect", "image.preprocess"};
            bool seeta = false;
            bool age_model_available = false;
#ifdef CVEDIX_WITH_FACE
            seeta = true;
            age_model_available = std::filesystem::is_regular_file("models/seetaface6/age_predictor.csta");
            tasks.push_back("face.recognize"); tasks.push_back("face.register"); tasks.push_back("face.delete");
#endif
            return {{"engine_commit", HERAMIND_ENGINE_COMMIT}, {"protocol", 1}, {"backend", "cpu"},
                {"qnn_available", false}, {"seetaface_compiled", seeta}, {"tasks", tasks},
                {"age_estimation", {{"supported", seeta}, {"model_available", age_model_available}, {"option", "estimate_age"}}},
                {"opencv", CV_VERSION}};
        }
        if (task == "image.inspect" || task == "image.preprocess") {
            auto frame = images.run(decode_image(args), {{"task", task}, {"options", options}});
            auto result = json::parse(frame->description);
            if (result.contains("error")) throw std::runtime_error(result["error"].get<std::string>());
            return result.at("data");
        }
        if (task == "face.recognize" || task == "face.register" || task == "face.delete") {
#ifdef CVEDIX_WITH_FACE
            ensure_faces();
            if (task == "face.delete") {
                auto id = args.at("face_id").get<int64_t>();
                if (id < 0 || !face_node->deleteFace(id)) throw std::runtime_error("Face ID was not found");
                if (!face_node->saveDatabase()) throw std::runtime_error("Face removed in memory, database save failed");
                return {{"deleted", true}, {"face_id", id}};
            }
            auto image = decode_image(args);
            // The detector cannot process degenerate pyramid sizes reliably.
            // Reject them before queueing a frame that may never reach the sink.
            if (image.cols < 40 || image.rows < 40)
                throw std::runtime_error("Face tasks require an image of at least 40x40 pixels");
            auto frame = faces->run(image, json::object());
            if (task == "face.register") {
                const auto name = args.at("name").get<std::string>();
                if (name.empty() || name.size() > 256 || name.find_first_of("\r\n\t") != std::string::npos)
                    throw std::runtime_error("Name must contain 1–256 bytes without control separators");
                if (frame->face_targets.size() != 1) throw std::runtime_error("Registration requires exactly one face");
                auto id = face_node->registerFace(image, name);
                if (id < 0) throw std::runtime_error("SeetaFace registration failed");
                if (!face_node->saveDatabase()) throw std::runtime_error("Face registered in memory, database save failed");
                return {{"face_id", id}, {"name", name}, {"backend", "cpu"}};
            }
            const bool estimate_age = options.value("estimate_age", false);
            if (estimate_age) ensure_age();
            json found = json::array();
            for (auto& face : frame->face_targets) {
                json item = {{"bbox", {face->x, face->y, face->width, face->height}},
                    {"detection_score", face->score}, {"identity", face->identify},
                    {"similarity", face->identify_score}, {"keypoints", face->key_points}};
                if (estimate_age) {
                    item["age_estimate"] = nullptr;
                    item["age_estimation_status"] = "missing_landmarks";
                    if (face->key_points.size() == 5) {
                        std::array<SeetaPointF, 5> points;
                        for (size_t i = 0; i < points.size(); ++i)
                            points[i] = {static_cast<double>(face->key_points[i].first), static_cast<double>(face->key_points[i].second)};
                        SeetaImageData simg;
                        simg.width = image.cols; simg.height = image.rows;
                        simg.channels = image.channels(); simg.data = image.data;
                        int age = -1;
                        if (age_predictor->PredictAgeWithCrop(simg, points.data(), age) && age >= 0) {
                            item["age_estimate"] = age;
                            item["age_estimation_status"] = "estimated";
                        } else item["age_estimation_status"] = "unavailable";
                    }
                }
                found.push_back(std::move(item));
            }
            return {{"faces", found}, {"backend", "cpu"}, {"model", "seetaface6"},
                {"age_estimation_enabled", estimate_age},
                {"width", image.cols}, {"height", image.rows}, {"engine_frame_id", frame->frame_index}};
#else
            throw std::runtime_error("SeetaFace was not built; rebuild with HERAMIND_WITH_SEETAFACE=ON and install models");
#endif
        }
        throw std::runtime_error("Unsupported engine task");
    }
};

int main() {
    // Preserve the pipe, then redirect both C and C++ library logs away from it.
    const int protocol_fd = dup(STDOUT_FILENO);
    if (protocol_fd < 0 || dup2(STDERR_FILENO, STDOUT_FILENO) < 0) return 2;
    cv::setNumThreads(2);
    CVEDIX_SET_LOG_LEVEL(cvedix_utils::cvedix_log_level::WARN);
    CVEDIX_SET_LOG_TO_FILE(false);
    CVEDIX_LOGGER_INIT();
    try {
        Engine engine;
        for (;;) {
            uint32_t network_length;
            if (!read_exact(&network_length, sizeof(network_length))) break;
            const size_t length = ntohl(network_length);
            if (!length || length > MAX_REQUEST) return 3;
            std::string bytes(length, '\0');
            if (!read_exact(bytes.data(), length)) return 3;
            json request, response = {{"protocol", 1}, {"request_id", nullptr}, {"ok", false}};
            try {
                request = json::parse(bytes);
                response["request_id"] = request.at("request_id");
                if (request.at("protocol") != 1 || !request.at("request_id").is_string())
                    throw std::runtime_error("Invalid protocol version or request ID");
                response["data"] = engine.execute(request.at("task").get<std::string>(), request.at("args"));
                response["ok"] = true;
            } catch (const std::exception& e) { response["error"] = e.what(); }
            auto encoded = response.dump();
            if (encoded.size() > MAX_RESPONSE) return 4;
            network_length = htonl(static_cast<uint32_t>(encoded.size()));
            if (!write_exact(protocol_fd, &network_length, sizeof(network_length))
                || !write_exact(protocol_fd, encoded.data(), encoded.size())) return 3;
        }
    } catch (const std::exception& e) { std::cerr << e.what() << '\n'; return 2; }
    close(protocol_fd);
}
