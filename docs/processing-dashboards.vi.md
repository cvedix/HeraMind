# Dashboard Face Recognition và OCR tài liệu

Tạo hai dashboard trong **Dashboard trực quan**, dùng các widget extension đã cài:

```bash
python3 /home/radxa/Documents/HeraMind/scripts/create-processing-dashboards.py --apply --web-url http://192.168.1.110:5173
```

Nhập tài khoản HeraMind trong terminal. Lệnh in liên kết đến từng dashboard.
Nếu dashboard cùng tên đã có, lệnh giữ nguyên bố cục và cấu hình, không tạo trùng.
Không tạo trang ứng dụng mới và không thay thế dashboard RTSP đã có.

## Nhận diện khuôn mặt - Cam2

Gồm widget **Face Recognition**, ba thẻ lượt xử lý/nhận ra/chưa biết và hướng dẫn.
Dùng thiết bị `cam2-rtsp`, metric ảnh `image`, cùng binding camera đã thiết lập.
Tiếp tục chạy `rtsp-face-feed.py` để cấp ảnh và đăng ký ảnh mẫu trong widget.
Xem [hướng dẫn Face Recognition](face-recognition.vi.md) cho runtime và mô hình cần thiết.
Các thẻ thống kê là lượt tổng của extension trên mọi thiết bị, không phải số người duy nhất.

## OCR tài liệu

Gồm widget **PaddleOCR v6**, trạng thái mô hình, tier và hướng dẫn xử lý.
Tải/kéo thả/dán ảnh JPG, PNG hoặc WebP, rồi bấm **Recognize**.
Widget hiển thị các vùng chữ, độ tin cậy và văn bản trích xuất.
Mặc định dùng tier `tiny` đã có sẵn, không cần camera RTSP hay đăng ký khuôn mặt.

Widget hiện xử lý từng ảnh. PDF cần xuất từng trang thành ảnh trước; chưa có nhập PDF trực tiếp
hoặc xử lý tài liệu hàng loạt. Bộ ký tự của tier `tiny` hiện chưa đầy đủ tiếng Việt có dấu;
không bảo đảm đọc đúng toàn bộ dấu tiếng Việt. Kiểm tra lại khoảng trắng và các trường số quan trọng.
Các thẻ mô hình là trạng thái chung của extension, cập nhật theo chu kỳ metric backend.

Chỉ tạo một dashboard bằng `--only face` hoặc `--only ocr`.
Đổi camera cho dashboard khuôn mặt bằng `--device-id TEN_THIET_BI` khi tạo mới.
