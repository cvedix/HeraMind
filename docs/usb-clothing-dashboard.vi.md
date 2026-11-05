# Dashboard camera USB: màu áo/quần và sự kiện

Tab **Camera USB - Màu áo và quần** dùng webcam USB đang cấp ảnh vào thiết bị
`cam2-rtsp`. Bên trái là ảnh đã phân tích, khung người và nhãn màu; bên phải
là danh sách tối đa 100 sự kiện, mới nhất trước, giờ Việt Nam.

Luồng xử lý: webcam → metric `image` → YOLO11n phát hiện người → SCHP ATR
phân vùng áo/quần → HSV/Lab đo màu → telemetry và widget `usb-clothing-camera`.
Không mở webcam lần thứ hai. Luồng nhận diện khuôn mặt tiếp tục dùng cùng nguồn ảnh.

- Dịch vụ user: `heramind-usb-clothing.service`, tự chạy cùng máy và thử lại khi lỗi.
- Bộ xử lý: `scripts/usb-clothing-events.py`, lấy ảnh mới mỗi 2,5 giây, tối đa 2 người/ảnh.
- Ảnh và kết quả màu luôn thuộc cùng khung hình; widget ghi rõ thời gian ảnh.
- Sự kiện màu lặp lại được hạn chế trong 10 giây; số người chỉ có ý nghĩa trong
  ảnh hiện tại, không phải tracking hoặc danh tính.
- Áo/quần không thấy rõ hoặc ảnh hồng ngoại có thể trả “Không rõ”. Không tạo
  sự kiện phát hiện người khi ảnh không có người đủ rõ.
- Sự kiện lưu trong metric `clothing_events`; ảnh phân tích và thông tin mô hình
  lưu trong `clothing_analysis`; tình trạng bộ xử lý ở `clothing_status`.
  Dữ liệu tuân theo chính sách telemetry của HeraMind.

Mô hình và runtime đã cài qua endpoint upload extension có xác thực. Mã nguồn
widget/worker và manifest nằm trong `extensions/clothing-colors/`. Script
`scripts/package-clothing-colors.py --skip-deps` đóng gói lại, bao gồm YOLO11n
đã cài tại `/var/lib/heramind/extensions/yolo-video/models/yolo11n.onnx`.

Bố cục camera RTSP trước khi đổi được sao lưu tại
`.local/usb-face-feed/camera-dashboard-before-clothing.json` (quyền 600).
Ảnh kiểm tra giao diện: `.local/usb-face-feed/clothing-dashboard.png`.

Kiểm tra đã thực hiện: 6 kiểm thử worker (mask màu, vùng thiếu, ảnh xám, lọc
người và NMS); kiểm thử tạo sự kiện và chống lặp; ảnh mẫu thật qua cả YOLO/SCHP;
trình duyệt kiểm tra ảnh webcam thật, danh sách sự kiện bằng fixture chỉ ở trình
duyệt, bố cục mobile và không có lỗi JavaScript. Không đưa sự kiện thử vào backend.

## Nguồn đang dùng: RTSP Cam2

Theo cấu hình mới, dashboard `dashboard_200fcc63-ca23-446a-a2d3-bc3615bf71f9`
đã đổi sang **Camera RTSP - Màu áo và quần**. Nguồn lấy từ tệp RTSP hiện có
`.local/camera-smoke/source`, không ghi URL/mật khẩu vào dashboard.

Hai dịch vụ user `heramind-rtsp-clothing-feed.service` (cấp ảnh mỗi 1 giây) và
`heramind-rtsp-clothing.service` (phân tích mỗi khoảng 2,5 giây, tùy tải) tự chạy
cùng máy. Thiết bị và telemetry riêng là `cam2-clothing-rtsp`; danh sách sự kiện
của camera RTSP tách khỏi sự kiện webcam trước đó. Bộ phân tích trang phục USB
đã tắt. Nhận diện khuôn mặt vẫn dùng webcam USB và không thay nguồn.

Đã kiểm tra ảnh RTSP thật, phát hiện người, tạo sự kiện thật, hiển thị canvas và
sự kiện bên phải trên trình duyệt. Ảnh kiểm tra nằm tại
`.local/usb-face-feed/clothing-rtsp-dashboard.png`.

## Tối ưu tải đã áp dụng

Phân tích trang phục hiện lấy mẫu mỗi 5 giây, RTSP cấp ảnh mỗi 2 giây. Dashboard
đọc lệnh `get_latest` của extension mỗi 2 giây (tab ẩn: 10 giây), giữ ảnh mới
nhất trong RAM. Sự kiện được lưu nguyên tử trong
`/var/lib/heramind/extensions/clothing-colors/data/cam2-clothing-rtsp-events.json`,
tối đa 100 bản ghi; không ghi lặp ảnh phân tích hoặc toàn bộ danh sách sự kiện
vào telemetry mỗi lượt. Các metric telemetry cũ chỉ còn là dữ liệu lịch sử.
Chi tiết số đo trong `docs/ai-performance.vi.md`.
