cmake_minimum_required(VERSION 3.16)
project(heramind_engine_worker LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

get_filename_component(DEFAULT_ENGINE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../thirdparty/engine" ABSOLUTE)
set(ENGINE_ROOT "${DEFAULT_ENGINE_ROOT}" CACHE PATH "CVEDIX engine checkout")
option(HERAMIND_WITH_SEETAFACE "Build the SeetaFace engine node" OFF)
set(SEETAFACE_ROOT "${ENGINE_ROOT}/third_party/seetaface6/build" CACHE PATH "SeetaFace installation")
find_package(OpenCV 4.6 REQUIRED COMPONENTS core imgproc imgcodecs dnn highgui videoio)
find_package(Threads REQUIRED)

# Only the input-processing core is needed. Do not pull in video servers,
# MediaToolKit, Kafka, CUDA or unrelated detectors from the full engine build.
file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/cvedix")
foreach(part nodes objects utils excepts third_party)
    file(CREATE_LINK "${ENGINE_ROOT}/${part}" "${CMAKE_CURRENT_BINARY_DIR}/cvedix/${part}" SYMBOLIC)
endforeach()
file(GLOB_RECURSE ENGINE_OBJECTS CONFIGURE_DEPENDS "${ENGINE_ROOT}/objects/*.cpp")
add_executable(engine-worker worker.cpp ${ENGINE_OBJECTS}
    "${ENGINE_ROOT}/nodes/common/cvedix_node.cpp"
    "${ENGINE_ROOT}/nodes/common/cvedix_meta_publisher.cpp"
    "${ENGINE_ROOT}/nodes/common/cvedix_meta_subscriber.cpp"
    "${ENGINE_ROOT}/utils/logger/cvedix_logger.cpp"
    "${ENGINE_ROOT}/utils/logger/cvedix_log_file_writer.cpp")
target_include_directories(engine-worker PRIVATE
    "${CMAKE_CURRENT_BINARY_DIR}" "${ENGINE_ROOT}" "${OpenCV_INCLUDE_DIRS}")
target_link_libraries(engine-worker PRIVATE ${OpenCV_LIBS} Threads::Threads stdc++fs)
execute_process(COMMAND git rev-parse --short HEAD WORKING_DIRECTORY "${ENGINE_ROOT}"
    OUTPUT_VARIABLE ENGINE_COMMIT OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process(COMMAND git diff --quiet WORKING_DIRECTORY "${ENGINE_ROOT}" RESULT_VARIABLE ENGINE_DIRTY)
if(ENGINE_DIRTY)
    string(APPEND ENGINE_COMMIT "-dirty")
endif()
target_compile_definitions(engine-worker PRIVATE HERAMIND_ENGINE_COMMIT="${ENGINE_COMMIT}")

if(HERAMIND_WITH_SEETAFACE)
    target_compile_definitions(engine-worker PRIVATE CVEDIX_WITH_FACE)
    target_include_directories(engine-worker PRIVATE "${SEETAFACE_ROOT}/include")
    target_sources(engine-worker PRIVATE
        "${ENGINE_ROOT}/nodes/infers/base/cvedix_infer_node.cpp"
        "${ENGINE_ROOT}/nodes/infers/base/cvedix_primary_infer_node.cpp"
        "${ENGINE_ROOT}/nodes/infers/cvedix_face_recognizer_node.cpp")
    foreach(module SeetaFaceDetector600 SeetaFaceLandmarker600 SeetaFaceRecognizer610 SeetaMaskDetector200 SeetaAgePredictor600 tennis)
        find_library(SEETA_${module} NAMES ${module} PATHS "${SEETAFACE_ROOT}/lib" NO_DEFAULT_PATH REQUIRED)
        target_link_libraries(engine-worker PRIVATE "${SEETA_${module}}")
    endforeach()
    set_target_properties(engine-worker PROPERTIES BUILD_RPATH "$ORIGIN/lib;${SEETAFACE_ROOT}/lib")
endif()
install(TARGETS engine-worker RUNTIME DESTINATION assets)
