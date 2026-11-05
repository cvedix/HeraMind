# Dashboard RTSP bằng các tính năng có sẵn

Dashboard này dùng `yolo-video-card`, bốn `value-card` và một `markdown-display`.
Không cần thêm trang hay component mới. Extension `yolo-video` phải được cài và đang chạy.

Tạo dashboard bằng API có sẵn, đăng nhập trong terminal:

```bash
python3 /home/radxa/Documents/HeraMind/scripts/create-rtsp-dashboard.py --apply --web-url http://192.168.1.110:5173
```

Lệnh dùng nguồn RTSP đã chuẩn bị trong `.local/camera-smoke/source` nếu tệp này có sẵn.
Trên máy khác, thêm `--rtsp-file /duong/dan/camera-url.txt` hoặc nhập URL khi được hỏi.
Mật khẩu nhập ẩn và không được lưu. Không đưa URL có khóa truy cập vào Git.
Nếu đã có dashboard cùng tên, lệnh giữ nguyên dashboard đó.

Giao diện ở cổng 5173 có hỗ trợ tên token NeoMind mà component YOLO hiện tại dùng.
Để cập nhật giao diện cài hệ thống ở cổng 9375 sau khi build:

```bash
sudo cp -a /home/radxa/Documents/HeraMind/web/dist/. /var/www/heramind/
```

Trong dashboard vừa tạo, bấm **Start / Bắt đầu** trên YOLO Video để phát luồng.
Dùng chế độ **Chỉnh sửa** của dashboard để kéo thả, thay đổi kích thước hoặc đổi cấu hình.
Trong YOLO Video, có thể vẽ ROI, chọn lớp đối tượng và vẽ đường đếm.
Các lớp COCO liên quan: person (0), bicycle (1), car (2), motorcycle (3), bus (5), truck (7).

Bốn thẻ số liệu lấy `current_fps`, `active_streams`, `total_frames_processed` và
`total_detections` từ extension YOLO Video. Đây là số liệu tổng của extension,
không tách riêng Cam2 nếu có nhiều luồng. Tổng lượt phát hiện không phải số người
duy nhất và không phải lượt đi qua đường đếm. Số liệu ROI/đường đếm hiển thị trong component YOLO.
Các thẻ lấy dữ liệu theo chu kỳ thu thập metric của backend (mặc định khoảng 60 giây);
thống kê ngay trên video cập nhật theo luồng.

Để bổ sung đăng ký và nhận diện khuôn mặt ngay trong dashboard này, xem
[tích hợp Face Recognition](face-recognition.vi.md).

Để tạo thêm hai dashboard chuyên biệt cho khuôn mặt và OCR ảnh tài liệu, xem
[dashboard xử lý Face Recognition và OCR](processing-dashboards.vi.md).
