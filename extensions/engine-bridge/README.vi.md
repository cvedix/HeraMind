# HeraMind Engine Bridge

Xem [hướng dẫn REST API](API.vi.md) để gọi xử lý ảnh, ước lượng tuổi,
đăng ký khuôn mặt và lấy kết quả từ ứng dụng bên ngoài.

Native extension `engine-bridge` nhận ảnh từ HeraMind và giao cho một worker
C++ dùng lõi `thirdparty/engine`. Worker tồn tại qua nhiều yêu cầu, có pipe
riêng do extension runner sở hữu; không mở cổng mạng hay tự đọc camera ngoài.

Luồng hiện tại:

```text
DeviceMetric hoặc command process
  → engine-bridge trong extension runner
  → worker C++: ingress node → image processor / SeetaFace → result node
  → kết quả kèm request_id, device_id, source_timestamp
  → command response / Custom event engine.result / cache get_latest
```

## Chức năng

Ví dụ có giao diện đã được khởi chạy trong dashboard **SeetaFace · Engine Demo**.
Để tạo hoặc chạy lại ví dụ trên HeraMind đang hoạt động:

```bash
python3 scripts/run-seetaface-demo.py --token-file /path/to/api-key-file
```

Script tạo thiết bị riêng `seetaface-demo`, giữ binding camera hiện có, gửi ảnh
mẫu qua telemetry và in URL dashboard. Kết quả phải đi qua engine thật trước
khi script báo thành công. Trên dashboard, có thể chạy ảnh mẫu, chọn ảnh khác,
xem khung mặt/5 mốc, đăng ký nhãn và xóa bản đăng ký vừa tạo. Ví dụ dùng ảnh
mẫu đi kèm SeetaFace, không giả làm camera trực tiếp. API key được đọc từ tệp;
không được ghi vào URL dashboard hoặc report.

Khi phát triển giao diện, chạy kiểm chứng trên browser:

```bash
node scripts/tests/verify-seetaface-demo.mjs
```

Phép thử thực hiện phát hiện, đăng ký/đối chiếu và xóa nhãn thử trên dashboard;
ảnh chụp được lưu tại `.local/seetaface-demo.png`, kết quả tại
`.local/seetaface-demo-ui-validation.json`.

| Task | Khả năng |
| --- | --- |
| `image.inspect` | Decode JPEG/PNG/WebP, kích thước, pixel format, mean BGR |
| `image.preprocess` | Resize tối đa 2048×2048; BGR hoặc grayscale; trả JPEG |
| `face.recognize` | Nhận diện qua node SeetaFace6 của engine, nếu build và có model |
| `face.register` / `face.delete` | Đăng ký/xóa trong DB SeetaFace riêng của extension |
| `health` | Báo backend thực tế, engine commit và các task đã build |

### Ước lượng độ tuổi

Gói model hiện gồm bảy tệp: sáu model nhận diện và `age_predictor.csta`.
Trong task `face.recognize`, thêm `"estimate_age": true` vào `options` để bật
SeetaFace AgePredictor. Mặc định API giữ chế độ nhận diện không ước lượng tuổi.
Binding của dashboard demo đã bật tùy chọn này:

```json
{"backend": "cpu", "estimate_age": true}
```

Mỗi khuôn mặt có `age_estimate` và `age_estimation_status`. Nếu model không
trả được kết quả, tuổi là `null`; ảnh không có mặt trả danh sách rỗng.
Nếu thiếu model khi đã yêu cầu bật tuổi, worker báo lỗi rõ ràng. Kiểm tra
`health.age_estimation.model_available` trước khi bật ở thiết bị khác.
Giao diện hiển thị **Tuổi ước lượng**, không coi kết quả là tuổi đã xác minh.

Phần mở rộng này chỉ thêm model độ tuổi. Không có suy đoán giới tính hay trạng
thái cảm xúc từ khuôn mặt; các thông tin đó có thể được cung cấp như dữ liệu
do người dùng tự khai báo trong một luồng riêng.

Đây là bản xử lý ảnh. Chưa cung cấp pipeline audio, LLM hoặc dữ liệu cảm biến
tùy ý. Không nhận URL, lệnh shell hay đường dẫn tệp do client truyền vào.
Worker dùng CPU; yêu cầu `options.backend = "qnn"` bị từ chối, không tự đổi
sang CPU. Chưa có model SeetaFace được chuyển sang QNN hoặc benchmark NPU.

## Build và đóng gói trên Q900

Cần Rust theo `rust-toolchain.toml`, CMake, compiler C++20, OpenCV development
4.6 trở lên và submodule engine đã khởi tạo. Cùng một workspace/lockfile giúp
extension và runner dùng thống nhất SDK ABI và cấu hình `serde_json`.

Từ thư mục HeraMind:

```bash
python3 scripts/build-engine-bridge.py
```

Script build C++, chạy kiểm thử transport và pipeline thật, build extension
release, rồi tạo `.local/engine-bridge/engine-bridge.nep`. Script không cài vào
backend đang chạy. Upload gói bằng chức năng cài extension sẵn có của HeraMind
hoặc endpoint có xác thực `POST /api/extensions/upload/file`.

Thêm `--runner /usr/local/bin/heramind-extension-runner` để kiểm tra cả ABI,
command, nhận `DeviceMetric` và publish `engine.result` qua runner thực tế.

Gói cần OpenCV system libraries trên thiết bị đích. Khi có SeetaFace build,
dùng `python3 scripts/build-engine-bridge.py --seetaface`; script yêu cầu các
thư viện tương ứng và đóng kèm chúng dưới `assets/lib`. Model chỉ được đóng kèm
khi truyền `--models-dir`; database danh tính không được đóng gói. `engine.patch` lưu các sửa đổi trong submodule cần
thiết để tái lập bản build; cần commit thay đổi ở repository engine và cập
nhật gitlink khi đưa tính năng vào nhánh phát hành.

Build/test nhanh khi phát triển:

```bash
cmake -S extensions/engine-bridge/native -B .local/engine-bridge-build -DCMAKE_BUILD_TYPE=Release
cmake --build .local/engine-bridge-build -j4
HERAMIND_TEST_ENGINE_WORKER="$PWD/.local/engine-bridge-build/engine-worker" python3 extensions/engine-bridge/native/test_worker.py
HERAMIND_TEST_ENGINE_WORKER="$PWD/.local/engine-bridge-build/engine-worker" cargo test -p heramind-engine-bridge -- --include-ignored
```

## Nhận ảnh tự động từ thiết bị

Cấu hình extension qua `PUT /api/extensions/engine-bridge/config` với JSON:

```json
{
  "timeout_ms": 10000,
  "bindings": [
    {
      "device_id": "cam2-rtsp",
      "metric": "image",
      "task": "image.inspect",
      "options": {"backend": "cpu"},
      "min_interval_ms": 1500,
      "enabled": true
    }
  ]
}
```

API lưu cấu hình và thử hot reload. Kiểm tra `hot_reloaded` trong phản hồi;
nếu false, reload extension. Mỗi lần cấu hình là thay toàn bộ danh sách,
tối đa 64 binding; một cặp device/metric chỉ được khai báo một lần.
`min_interval_ms` giới hạn nhịp nhận ảnh riêng cho mỗi binding (0–60000 ms,
mặc định 0). Ảnh đến sớm hơn nhịp này bị bỏ và được tính vào `dropped`.

Extension chỉ xử lý metric đã binding, có giá trị ảnh dạng chuỗi. Chấp nhận
payload `DeviceMetric` trực tiếp hoặc envelope của event bus. Bỏ qua metric
ảo và cấm binding `virtual.engine.*` để ngăn kết quả quay lại thành đầu vào.

Mỗi ảnh được xử lý thành công sẽ phát Custom event `engine.result` qua
capability `event_publish`. Cache kết quả vẫn còn nếu việc publish lỗi;
`publish_failed` ghi nhận số lần lỗi. Các trường ảnh đầu ra không được đưa
vào event/cache để tránh khuếch đại payload. Muốn nhận JPEG đã resize, dùng
command trực tiếp. Không ghi lại ảnh đầu vào vào telemetry từ extension này.

## Gọi xử lý qua API

Gửi JSON đến `POST /api/extensions/engine-bridge/command` với phiên xác thực
HeraMind như các command extension khác:

```json
{
  "command": "process",
  "args": {
    "device_id": "camera-01",
    "source_timestamp": 1793900000000,
    "task": "image.preprocess",
    "image_base64": "data:image/jpeg;base64,...",
    "options": {"backend": "cpu", "width": 640, "height": 360, "format": "bgr"}
  }
}
```

Các command khác: `health`, `get_status`, `get_latest` (args `device_id`),
`face_register` (args `name`, `image_base64`) và `face_delete` (args `face_id`).
`get_status` cho biết cấu hình, lỗi cuối và counters. `get_latest` chỉ trả
kết quả mới nhất cho từng camera, không phải lịch sử hoặc thống kê người duy nhất.

Kết quả process có `request_id`, `device_id`, `source_timestamp`, thời điểm
nhận/hoàn thành, latency trong extension và `result`. Latency này không tính
thời gian camera chụp hoặc thời gian trước khi sự kiện đến extension.

## Bật SeetaFace

Tải bảy model từ liên kết trong
[repository chính thức](https://github.com/SeetaFace6Open/index), kiểm tra
checksum và build bản ARM64:

```bash
python3 scripts/setup-seetaface-models.py
SEETA_BUILD_JOBS=6 bash thirdparty/engine/third_party/seetaface6/build_seetaface6.sh
python3 scripts/build-engine-bridge.py --seetaface --models-dir .local/seetaface-models --runner /usr/local/bin/heramind-extension-runner
```

Model được lưu ở `.local/seetaface-models/`, archive ở
`.local/seetaface-downloads/`. `provenance.json` ghi nguồn, kích thước và
SHA-256 từng model. Checksum archive được pin từ bản tải chính thức đã kiểm tra;
đây không phải checksum do upstream công bố. `--models-dir` đóng kèm bảy model
đã xác minh vào gói để cài qua API, đồng thời chạy thử đăng ký/nhận lại/xóa với
ảnh mẫu SeetaFace trong database tạm.

Cài và chạy qua HeraMind bằng API key lưu trong tệp riêng:

```bash
python3 scripts/install-engine-bridge.py --package .local/engine-bridge/engine-bridge.nep --token-file /path/to/api-key-file --device cam2-rtsp --fixture thirdparty/engine/third_party/seetaface6/FaceRecognizer6/example/1.png
```

Lệnh chỉ cài extension `engine-bridge`, chạy thử ảnh mẫu qua API, rồi cấu hình
binding được chỉ định. Nó không tự đăng ký danh tính từ camera.
Nếu camera chưa cấp ảnh, lệnh báo `waiting_for_camera` và giữ binding để nhận
ảnh khi camera kết nối lại.

Trên Q900 hiện tại, bản có model đã được cài tại
`/var/lib/heramind/extensions/engine-bridge/` và kiểm thử qua API HeraMind.
Luồng REST telemetry → `DeviceMetric` → SeetaFace → `engine.result` đã đạt với
thiết bị thử riêng; thiết bị thử đã được xóa và cấu hình cũ được khôi phục.
Binding vận hành là `cam2-rtsp:image`, task `face.recognize`, CPU,
`min_interval_ms=1500`. Camera USB hiện ngắt kết nối nên chưa có ảnh live mới.
Trạng thái và kết quả kiểm chứng được lưu tại `.local/engine-bridge-live-status.json`.

Build các thư viện SeetaFace6 tại
`thirdparty/engine/third_party/seetaface6/build` trước khi dùng `--seetaface`.
Build ARM64 cần TenniS ARM/NEON, không dùng Haswell/AVX hoặc CUDA.
Chuẩn bị các model sau trong thư mục cài extension `models/seetaface6/`:

```text
face_detector.csta
mask_detector.csta
face_landmarker_pts5.csta
face_recognizer.csta
face_landmarker_mask_pts5.csta
face_recognizer_mask.csta
age_predictor.csta
```

Đổi binding/task thành `face.recognize`. Worker dùng SeetaFace SYNC, model
5 mốc và threshold 0.70 hiện có của engine. Cần hiệu chỉnh ngưỡng trên dữ liệu
thực trước sử dụng vận hành. `health.seetaface_compiled` chỉ nói thư viện đã
được build; việc nạp model được kiểm tra khi gọi tác vụ face đầu tiên.

Đăng ký yêu cầu ảnh có đúng một khuôn mặt; tên tiếng Việt có khoảng trắng
được giữ khi lưu/tải lại. Database nằm tại `state/seetaface/faces.*`, khác với
database InsightFace hiện có. Không đổi hoặc ghi đè gói `face-recognition`,
dashboard, model và danh tính đang sử dụng. Embedding của hai model không
thay thế cho nhau; người dùng cần đăng ký vào kho SeetaFace riêng.

Nhánh SeetaFace đã chạy inference CPU trên Q900 bằng model chính thức: phát
hiện, đăng ký, đối chiếu, giữ nguyên tên tiếng Việt sau restart, xóa và kiểm
tra trạng thái xóa sau restart. Các đăng ký thử nằm trong database tạm rồi
được dọn, không được đưa vào database vận hành. Báo cáo được lưu trong
`.local/engine-bridge/package/seetaface-validation.json` sau build có model.
Các số đo trong báo cáo là phép thử một ảnh mẫu, không phải benchmark tải camera.

Nhánh xử lý ảnh CPU đã qua 5 kiểm thử worker, 7 kiểm thử Rust gồm roundtrip
C++ và phép thử runner với capability hai chiều trên Q900. Chuyển mô hình
sang QNN/NPU chưa được thực hiện. Chi tiết hướng NPU tại
[báo cáo thiết kế](../../docs/engine-seetaface-qualcomm.vi.md).

## Giới hạn tài nguyên và khôi phục

Một worker xử lý một yêu cầu mỗi lần; yêu cầu đồng thời bị báo busy và tăng
`dropped`, không xếp hàng vô hạn trong extension. Điều này chưa thay thế chính
sách hàng đợi của event bus; nên đặt chu kỳ lấy ảnh phù hợp ở thiết bị nguồn.
Tối đa request 8 MiB, chuỗi base64 ảnh 6 MiB, phản hồi 2 MiB, ảnh decode
16 megapixel và mỗi chiều tối đa 8192. JPEG đầu ra tối đa 1 MiB.
Endpoint command của HeraMind giới hạn `args` ở 1 MiB, chặt hơn giao thức worker;
giới hạn 6 MiB không áp dụng cho ảnh gửi qua endpoint command này.

Timeout 100–60000 ms, mặc định 10 giây. Timeout, EOF, phản hồi quá lớn, sai
request ID hoặc future bị hủy sẽ khiến worker bị hủy/khởi tạo lại trước yêu
cầu tiếp theo. Lỗi dữ liệu hợp lệ ở cấp giao thức không buộc nạp lại engine.
Khởi động lại không tự thử lại thao tác đăng ký/xóa vì thao tác có thể đã
thực hiện trước khi mất phản hồi. Khi xảy ra lỗi lưu DB sau mutation, worker
báo rõ trạng thái trong bộ nhớ có thể đã thay đổi; cần kiểm tra trước thử lại.
