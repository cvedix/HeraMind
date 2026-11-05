# Kiểm tra tải AI ngày 28/10/2026

Đo trên máy 8 CPU logic trong cửa sổ khoảng 12 giây, khi camera hoạt động.
CPU bên dưới dùng quy ước 100% = 1 lõi; là quan sát ngắn hạn, không phải tải tối đa.

| Tiến trình | CPU đo | RAM RSS |
|---|---:|---:|
| Face Recognition | 448,4% | 435 MB |
| Clothing Colors: YOLO + SCHP | 211,5% | 430 MB |
| Backend | 6,4% | 457 MB |
| LLM MiniCPM | 0% trong lúc không có yêu cầu | 4.057 MB |
| OCR | 0% trong lúc không có yêu cầu | 624 MB |
| YOLO extension riêng | 0% trong lúc không có yêu cầu | 189 MB |

Nhận diện khuôn mặt đang nhận yêu cầu mỗi 0,1 giây. Nhật ký ghi nhận
`IPC event channel full ... dropping event` ở extension face-recognition.
Phân tích màu mất khoảng 2,2–2,3 giây/ảnh ở cấu hình cũ, chu kỳ 2,5 giây.
API current trung vị 13,5 ms, cao nhất 42,9 ms trong mẫu đo.
Máy còn khoảng 26 GiB RAM khả dụng, đĩa trống khoảng 19 GiB.

## Thay đổi đã chuẩn bị và kiểm thử

- Webcam nhận diện khuôn mặt: mặc định 1 giây/ảnh; sàn 0,5 giây để chặn quá tải
  khi thanh chỉnh yêu cầu 0,0–0,4 giây. 0,0 nghĩa nhanh nhất trong giới hạn này.
- Bộ cấp RTSP: mỗi 2 giây; bộ phân tích trang phục: mỗi 5 giây, luôn nghỉ ít
  nhất 1 giây sau khi xử lý, không tạo yêu cầu suy luận đồng thời.
- Bộ cấp JPEG chỉ giữ một khung mới nhất, bỏ khung cũ khi xử lý chậm.
- SCHP dùng 2 luồng CPU, YOLO trong worker dùng 1 luồng, OpenCV dùng 1 luồng.
- Snapshot trang phục mới nhất giữ trong RAM của worker; không tiếp tục ghi
  ảnh kèm toàn bộ kết quả vào lịch sử telemetry mỗi lượt.
- Giữ tối đa 100 sự kiện/camera, ghi tệp nguyên tử trong extension data khi
  có sự kiện mới; không ghi ảnh vào tệp sự kiện. Bảo toàn dữ liệu lịch sử cũ.
- Dashboard đọc snapshot mỗi 2 giây, tab ẩn mỗi 10 giây, không chồng yêu cầu.
- Giảm log và chỉ ghi trạng thái telemetry khi trạng thái thay đổi.

Kiểm thử: 14 kiểm thử scripts, 7 kiểm thử worker, build Rust release và kiểm
tra cú pháp Python/JavaScript đạt. Kiểm thử snapshot xác nhận chỉ giữ ảnh mới
nhất, giới hạn 100 sự kiện, khôi phục sự kiện sau restart và không lưu ảnh vào
JSON sự kiện.

## Đã triển khai và đo lại

Backend đã được khởi động lại và hai extension đã cập nhật thành công.
Script triển khai nay chờ API và cả hai extension sẵn sàng tối đa 120 giây,
tránh lỗi Connection refused ngay sau systemctl restart. Đã thêm 3 kiểm thử
cho kết nối từ chối tạm thời, chờ extension, lỗi xác thực và thời hạn chờ.

Mẫu đo lại khoảng 13,5 giây, có hai luồng camera hoạt động:

| Tiến trình | CPU trước | CPU sau |
|---|---:|---:|
| Face Recognition | 448,4% | 146,5% |
| Clothing Colors | 211,5% | 64,4% |
| Backend | 6,4% | 3,9% |

LLM/OCR/YOLO riêng ở trạng thái nhàn rỗi trong mẫu đo sau cùng. Một mẫu ngay
sau khởi động ghi nhận LLM có tải tạm thời; không suy diễn mẫu nhàn rỗi thành
cam kết tải khi có yêu cầu LLM/OCR đồng thời. RAM khả dụng khoảng 27 GiB.
API current trung vị 7,7 ms, cao nhất 37,5 ms. Phân tích màu khoảng 2,0–2,2
giây/ảnh, chu kỳ 5 giây nên có khoảng nghỉ. Không ghi nhận cảnh báo IPC event
channel full mới từ khi cập nhật đến lúc kiểm tra.

Ảnh đã phân tích mới nhất và tối đa 100 sự kiện được xác nhận trên trình duyệt
RTSP thật; bố cục mobile và JavaScript đều đạt. Dữ liệu sự kiện được khôi phục
từ tệp khi worker restart; không lưu ảnh vào tệp sự kiện. Lịch sử telemetry cũ
và chính sách retention hiện tại không bị thay đổi. Các ảnh camera đầu vào vẫn
tuân theo retention 72 giờ; việc ngừng ghi ảnh JSON lặp chỉ áp dụng cho kết quả
phân tích trang phục mới.

Cả ba dịch vụ user đều đang hoạt động: `heramind-usb-face`,
`heramind-rtsp-clothing-feed`, `heramind-rtsp-clothing`.

Số đo thô: `.local/ai-load-before.json`, `.local/ai-load-after.json`; script đo:
`.local/measure-ai-load.py`. Đây là các mẫu đo ngắn, cần theo dõi thêm khi số
camera hoặc số yêu cầu LLM/OCR đồng thời tăng.
