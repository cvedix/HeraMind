# Nhận diện khuôn mặt trong dashboard RTSP

Tích hợp dùng extension và widget `face-recognition-card` có sẵn, ngay trong dashboard
**Giám sát RTSP - Cam2**. Không thêm trang riêng. Camera RTSP được lấy mẫu JPEG, gửi
vào metric `image` của thiết bị `cam2-rtsp`; Face Recognition tự nhận ảnh qua binding thiết bị.

## Cài runtime và mô hình

Sau khi backend đã build và mô hình đã chuẩn bị trong `.local/face-models`, chạy:

```bash
sudo bash /home/radxa/Documents/HeraMind/scripts/install-face-runtime.sh
```

Lệnh cài mô hình còn thiếu, cập nhật binary và giao diện, rồi khởi động lại dịch vụ.
Backend mới truyền cả biến `HERAMIND_EXTENSION_DIR` và `NEOMIND_EXTENSION_DIR`
cho extension để tìm mô hình và lưu cơ sở dữ liệu khuôn mặt đúng vị trí.

Extension cần `det_10g.onnx` (phát hiện) và `w600k_r50.onnx` (ArcFace, đối chiếu danh tính).
Gói mô hình được lấy từ [InsightFace Model Zoo](https://github.com/deepinsight/insightface/blob/master/model_zoo/README.md)
và [bản phát hành chính thức](https://github.com/deepinsight/insightface/releases/tag/model-zoo).
Điều kiện sử dụng mô hình theo nhà cung cấp: các mô hình công khai dành cho nghiên cứu phi thương mại.

## Thêm vào dashboard đang có

```bash
python3 /home/radxa/Documents/HeraMind/scripts/create-rtsp-dashboard.py --apply --with-faces --face-feed
```

Đăng nhập bằng tài khoản HeraMind trong terminal. Nếu dashboard đã có, script chỉ nối thêm
widget khuôn mặt và hai thẻ số liệu, không thay thế các widget hiện hữu.
Thiết bị camera được tạo nếu chưa có; binding `image` đã có sẽ được dùng lại.
Thêm `--web-url http://192.168.1.110:5173` nếu muốn dùng giao diện phát triển.
Cờ `--face-feed` chạy bộ cấp ảnh ngay sau khi tạo dashboard, dùng phiên đăng nhập vừa nhập
mà không lưu token ra tệp; nhấn Ctrl+C để dừng cấp ảnh.

## Cấp ảnh và đăng ký khuôn mặt

```bash
python3 /home/radxa/Documents/HeraMind/scripts/rtsp-face-feed.py
```

Lệnh lấy một ảnh mỗi 5 giây từ nguồn RTSP đã cung cấp, chạy đến khi nhấn Ctrl+C.
Các ảnh được ghi vào telemetry của thiết bị và tuân theo chính sách lưu dữ liệu hiện tại.
Dùng `--interval 2` để thay chu kỳ, `--once` để thử một ảnh, `--rtsp-file` để đổi nguồn.
Không lưu mật khẩu đăng nhập. Khi token hết hạn, chạy lại và đăng nhập lại.

Trong widget **Nhận diện khuôn mặt**, mở phần đăng ký, tải ảnh có một khuôn mặt rõ nét
và nhập tên. Những lần sau widget hiển thị tên đã khớp hoặc khuôn mặt chưa biết.
Có thể xem và xóa bản đăng ký, tắt hoặc bật liên kết camera ngay trong widget.
Không tự đăng ký hay gán tên cho người xuất hiện trong luồng.

Hai thẻ thống kê là tổng lượt nhận ra/chưa biết của extension trên mọi thiết bị,
có thể đếm lặp cùng người qua nhiều ảnh; không phải số người duy nhất.

## Webcam USB tự kết nối

Bộ cấp ảnh hỗ trợ `--usb-device /dev/v4l/by-id/...-video-index0` thay cho RTSP.
Máy hiện tại dùng webcam Logitech `usb-046d_0825_1B70F150-video-index0`,
chụp 640×480, mặc định cấp ảnh mỗi 0,5 giây vào binding `cam2-rtsp` đang có.
Không chạy đồng thời bộ cấp RTSP vào cùng thiết bị này.

Dịch vụ user `heramind-usb-face.service` đã được cấu hình tự chạy khi khởi động
(user radxa có Linger=yes), thử lại khi rút/cắm USB hoặc backend khởi động lại.
Bản cấu hình nằm ở `deploy/heramind-usb-face.service`; các đường dẫn dành cho máy hiện tại.
Cần API key còn hiệu lực trong `.local/usb-face-feed/token`, quyền 600;
không dùng token đăng nhập ngắn hạn cho dịch vụ chạy thường xuyên.
Sau khi lưu key:

```bash
chmod 600 .local/usb-face-feed/token
systemctl --user restart heramind-usb-face.service
journalctl --user -u heramind-usb-face.service -n 20 --no-pager
```

Nếu thiếu tệp key, systemd bỏ qua việc chạy để tránh yêu cầu đăng nhập tương tác.
Dừng tự cấp ảnh bằng `systemctl --user disable --now heramind-usb-face.service`.

Widget webcam đã được sửa để dùng ảnh metric gốc khi extension chưa có ảnh
chú thích (ví dụ không có khuôn mặt trong khung hình). Bản frontend đã triển khai
và ghi chú nằm trong `extensions/face-recognition/`. Mô hình và trạng thái nhận diện
vẫn do backend xử lý; ảnh xem trước không tự tạo kết quả nhận diện.

### Chỉnh chu kỳ cập nhật

Trong widget webcam, thanh **Chu kỳ cập nhật ảnh** hỗ trợ 0,0–1,0 giây,
bước 0,1 giây, mặc định 0,5 giây. Giá trị được lưu trong metric
`image_update_interval` của thiết bị camera, áp dụng cho bộ cấp ảnh và chu kỳ
làm mới widget. Bộ cấp ảnh đọc thay đổi trong khoảng 1 giây.
0,0 nghĩa là nhanh nhất (tối đa 10 ảnh/giây); tốc độ thực tế phụ thuộc camera,
backend và thời gian nhận diện. Không có nghĩa xử lý ảnh tức thời.
