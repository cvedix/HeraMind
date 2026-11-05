# Mô hình nhận diện màu áo và quần cho HeraMind YOLO

Kiểm tra ngày 28/10/2026 trên máy hiện tại: Linux ARM64/aarch64, ONNX Runtime CPU 1.31.1,
4 luồng xử lý. Chưa sửa hoặc thay thế extension `yolo-video` đang chạy.

## Hai model đã tải và chạy thử

| Model | Chức năng | Kích thước tệp đo được | Thời gian đo |
|---|---|---:|---:|
| SCHP ATR, ONNX INT8 static | Phân vùng áo, quần và các phần khác | 69.141.996 byte | Trung vị warm 482,85 ms, 512×512, 3 lượt sau warm-up |
| MobileNetV3-Small distilV4 | Phân loại 13 màu cho vùng quần áo | 6.134.359 byte | Trung vị warm 6,08 ms, 224×224, 10 lượt sau warm-up |

Các phép đo là inference CPU riêng lẻ, không gồm YOLO, lấy RTSP, cắt người hay hậu xử lý.
Đây không phải benchmark toàn hệ thống hoặc cam kết FPS. Máy còn đang chạy các dịch vụ khác.

### SCHP ATR + đo màu trong mask: hướng thử nghiệm ưu tiên

[Model card của bản ONNX](https://huggingface.co/pirocheto/schp-atr-18),
[mã nguồn SCHP gốc](https://github.com/GoGoDuck912/Self-Correction-Human-Parsing).

- Model cung cấp vùng `Upper-clothes` (4), `Pants` (6), `Skirt` (5), `Dress` (7).
- Model **không trực tiếp trả tên màu**. Sau segmentation cần lấy màu từ pixel của từng vùng,
  loại nền/da, rồi gom màu bằng HSV hoặc clustering Lab.
- Input ONNX: `pixel_values`, float32 `[N,3,512,512]`; output `logits`, `[N,18,512,512]`.
- Tiền xử lý theo bộ export: RGB, resize bilinear 512×512, chia 255,
  mean `[0.406,0.456,0.485]`, std `[0.225,0.224,0.229]`, chuyển NCHW.
- Đã chạy trên ảnh demo của tác giả. Mask có vùng áo; ảnh demo chỉ có phần thân trên nên không có vùng quần.
  Chưa đánh giá độ chính xác màu áo/quần trên luồng Cam2.
- Bản export là bên thứ ba; repository đóng gói đã được lưu trữ read-only.
  Nên ghim revision và tự kiểm tra với ảnh camera thực trước khi dùng lâu dài.

Revision đã tải: `61f4ff3c508b899904e495cefbbaff9cacadeb45`.

### MobileNetV3-Small: phương án nhẹ cần đánh giá ảnh thật

[Model card của tác giả](https://huggingface.co/giosanchez0208/clothing-color-recognition).

- Tệp `distilV4_student.onnx`, input `image` `[N,3,224,224]`, output `color_logits` `[N,13]`.
- Nhãn: red, orange, yellow, green, blue, violet, purple, white, gray, black, pink, brown, olive.
- Phải xác định riêng vùng áo và vùng quần. Theo tác giả, input là crop ngữ cảnh 224×224 có vùng
  quần áo 112×112 ghép ở giữa; resize ảnh thông thường không đúng cách huấn luyện.
- Đã kiểm tra chạy được và thời gian bằng tensor giả. **Chưa kiểm tra dự đoán màu trên ảnh thật.**
- Tác giả công bố huấn luyện trên dữ liệu tổng hợp, chưa có đánh giá chuyển sang ảnh thực ở quy mô lớn.
  Vì vậy tốc độ nhanh không đồng nghĩa với độ chính xác phù hợp cho camera.

Revision đã tải: `a2650675ea2af6fbb01a94e66432774a6e0381bf`.

## Phương án khác

[Person-Attribute-Recognition-MarketDuke](https://github.com/Yukun-Huang/Person-Attribute-Recognition-MarketDuke)
có checkpoint và nhãn trực tiếp cho màu áo/quần. Cần tải checkpoint, kiểm tra điều kiện sử dụng,
chuyển sang ONNX và đo lại. Chưa chạy model này trên máy hiện tại.

Không chọn PP-Human PA100K mặc định cho yêu cầu màu: [danh sách thuộc tính chính thức](https://github.com/PaddlePaddle/PaddleDetection/blob/release/2.9/docs/advanced_tutorials/customization/pphuman_attribute_en.md)
có kiểu áo/quần và họa tiết, nhưng không có nhãn màu áo/quần.

## Điểm tích hợp vào extension

Luồng đề xuất:

```text
RTSP → YOLO phát hiện person → cắt từng người
     → phân vùng áo/quần → phân tích màu → metadata và nhãn trên widget YOLO hiện có
```

Mã upstream hiện tại nằm tại
[camthink-ai/NeoMind-Extensions/extensions/yolo-video/src](https://github.com/camthink-ai/NeoMind-Extensions/tree/main/extensions/yolo-video/src).
`Detection`/`ObjectDetection` hiện mang class, confidence và bbox; cần mở rộng metadata và cách vẽ nhãn
để bổ sung màu. Không thể đổi tên model mới thành `yolo11n.onnx` rồi thay vào: tensor đầu ra khác nhau.

Với SCHP, nên xử lý mẫu định kỳ ở worker riêng, hàng đợi có giới hạn, để không chặn luồng video.
Chưa có đo hiệu năng khi chạy đồng thời YOLO + SCHP. ID detection hiện được đánh số lại từng khung hình;
muốn lưu kết quả theo người qua nhiều khung hình cần thêm tracker ổn định.

Kết quả có thể gồm `upper_color`, `lower_color`, tỷ lệ màu chiếm ưu thế và thời điểm phân tích.
Nếu không nhìn thấy quần/áo, vùng quá nhỏ, bị che hoặc camera hồng ngoại ban đêm, trả `unknown`.
Không suy màu quần từ vùng da/chân hoặc nửa dưới váy. Ánh sáng và cân bằng trắng có thể làm lệch màu.

## Tệp thử nghiệm cục bộ

- `.local/clothing-research/schp-atr-18-int8-static.onnx`
- `.local/clothing-research/distilV4_student.onnx`
- `.local/clothing-research/benchmark.json`
- `.local/clothing-research/color-benchmark.json`
- Môi trường thử: `.local/clothing-venv` (không sửa Python hệ thống).

Các tệp model và ảnh thử nằm trong `.local`, không đưa vào Git. Chúng chưa được cài vào extension hệ thống.
