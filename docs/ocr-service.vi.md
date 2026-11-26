# OCR Service cho PDF quét

Mở dịch vụ tại `/ocr`, chọn tệp PDF và ngôn ngữ rồi bấm **Bắt đầu OCR**. Trang hiển thị tiến độ và lịch sử xử lý. Khi hoàn tất, chọn từng trang để đọc hoặc tải toàn bộ văn bản dưới dạng TXT.

Trong **Tài liệu gần đây**, bấm **Xoá** bên cạnh tài liệu đã hoàn tất hoặc bị lỗi rồi xác nhận để xoá mục lịch sử và văn bản nhận dạng khỏi máy chủ.

Để xoá nhiều tài liệu, đánh dấu từng mục hoặc bấm **Chọn tất cả (trang hiện tại)**, sau đó bấm **Xoá đã chọn** và xác nhận một lần. Chỉ tài liệu đã hoàn tất hoặc bị lỗi mới có thể được chọn.

HeraMind dùng Poppler để kết xuất ảnh theo trang. Chọn **Tesseract** hoặc **PaddleOCR-VL** trong trường **Mô hình OCR**. Đường dẫn `/ocr/paddle-vl` mở dashboard với PaddleOCR-VL được chọn sẵn. API trả văn bản UTF-8 theo thứ tự trang. Chất lượng phụ thuộc vào độ nét, góc nghiêng, bố cục và ngôn ngữ của bản quét.

## Kết nối hệ thống đối tác

Trong mục **Tích hợp API** tại `/ocr`, quản trị viên mở **Quản lý khóa OCR**, nhập tên đối tác và tạo khóa. Sao chép giá trị khóa ngay khi tạo và lưu vào kho bí mật của hệ thống gọi API. Khi xoay vòng khóa, tạo khóa mới trước, rồi thu hồi khóa cũ khi đối tác đã chuyển đổi. Dùng cùng một khóa để gửi tài liệu và đọc kết quả. Quản trị viên HeraMind có thể xem các job để theo dõi vận hành.

Thay `HERA_BASE` bằng địa chỉ HeraMind mà hệ thống gọi API truy cập được. Chạy ví dụ bằng Bash và nhập khóa OCR khi được hỏi.

```bash
HERA_BASE='http://127.0.0.1:9375'
read -rsp 'OCR_KEY: ' OCR_KEY
printf '\n'
```

### Kiểm tra dịch vụ

```bash
curl -fsS "$HERA_BASE/api/ocr/status?engine=paddle_ocr_vl" \
  -H "X-OCR-Key: $OCR_KEY"
```

Gửi tệp khi `data.engine_ready` là `true` và `data.languages` chứa ngôn ngữ bạn chọn. Phản hồi cũng cho biết `data.max_bytes`, `data.max_pages` và `data.retention_seconds`.

### Gửi PDF

```bash
curl -sS -i -X POST \
  "$HERA_BASE/api/ocr/jobs?engine=paddle_ocr_vl&language=vie_eng&filename=hoso.pdf" \
  -H "X-OCR-Key: $OCR_KEY" \
  -H 'Content-Type: application/pdf' \
  --data-binary @hoso.pdf
```

Gửi byte PDF trực tiếp trong thân request. `filename` là tên hiển thị tùy chọn, có thể mã hóa URL để dùng tên có dấu. `language` nhận `vie` cho tiếng Việt, `eng` cho tiếng Anh hoặc `vie_eng` cho tài liệu song ngữ. Mặc định dùng `vie_eng`.

`engine=paddle_ocr_vl` chuyển các trang tới backend PaddleOCR-VL. Backend dùng GPU NVIDIA khi máy chủ cấu hình `PADDLE_DEVICE=gpu:0`. Máy chủ quyết định thiết bị suy luận. Với PaddleOCR-VL, lựa chọn ngôn ngữ là gợi ý cho extension. Trạng thái `engine_ready=true` và `model_loaded=false` cho biết backend có thể nhận yêu cầu và sẽ nạp mô hình khi xử lý. `tesseract` là giá trị mặc định nếu bỏ qua `engine`.

Phản hồi `202 Accepted` trả mã job tại `data.id` và đường dẫn truy vấn trong header `Location`.

### Lấy trạng thái và kết quả

```bash
JOB_ID='your-job-id'
curl -fsS "$HERA_BASE/api/ocr/jobs/$JOB_ID" -H "X-OCR-Key: $OCR_KEY"
```

Thăm dò vài giây một lần trong khi `data.job.status` là `queued` hoặc `running`. `data.job.engine` cho biết mô hình đã chọn. Sau khi hoàn tất, đọc `data.pages`, gồm các phần tử có `number` (bắt đầu từ 1) và `text`. Nếu job chuyển sang `failed`, đọc `data.job.error_code`. Ví dụ `UNREADABLE_PDF` với PDF hỏng hoặc được bảo vệ bằng mật khẩu, `TOO_MANY_PAGES` khi vượt giới hạn trang và `PADDLE_INFERENCE_ERROR` khi extension báo lỗi suy luận.

`GET /api/ocr/jobs` liệt kê tối đa 50 job gần nhất mà khóa có quyền xem. `GET /api/ocr/status` trả trạng thái OCR, ngôn ngữ và giới hạn hiện hành. Có thể xem schema tại `/api/docs/openapi.json`.

`DELETE /api/ocr/jobs/{id}` xoá tác vụ đã kết thúc cùng văn bản nhận dạng. Dùng cùng khóa OCR hoặc phiên HeraMind có quyền xem tác vụ. API trả `409` khi tác vụ vẫn đang chờ hoặc đang xử lý.

Khi job đã hoàn tất, dùng `jq` để in văn bản theo thứ tự trang.

```bash
curl -fsS "$HERA_BASE/api/ocr/jobs/$JOB_ID" -H "X-OCR-Key: $OCR_KEY" |
  jq -er '.data | select(.job.status == "completed") | .pages[].text'
```

## Giới hạn vận hành

- Mỗi PDF tối đa 20 MiB và 40 trang. Hệ thống nhận tối đa 8 lượt gửi hoặc job đang chờ xử lý, với 2 job chạy đồng thời. Mỗi lần có tối đa một job PaddleOCR-VL chạy. Khi nhận `429`, đợi rồi thử lại.
- Kết quả có thời hạn tối đa 24 giờ kể từ lúc hoàn tất. Khi chạm ngưỡng 200 job, hệ thống dọn các job đã kết thúc cũ nhất. Sau khi job hết hiệu lực, truy vấn trả `404`.
- Tệp PDF và ảnh trang được lưu tạm trong lúc xử lý. HeraMind lưu trạng thái và văn bản nhận dạng trong thư mục dữ liệu của máy chủ.

## Giám sát request

Quản trị viên mở **OCR Monitor** trong **Dashboard trực quan** để xem lưu lượng, mã HTTP, thời gian phản hồi và mã tác vụ. Bộ lọc thời gian, endpoint và nhóm mã HTTP áp dụng cho các thẻ thống kê, biểu đồ và bảng request. Thời gian phản hồi P95 được tính trên các request khớp bộ lọc.

Monitor ghi các request `POST /api/ocr/jobs`, `GET /api/ocr/status`, `GET /api/ocr/jobs` và `GET /api/ocr/jobs/{id}`. Máy chủ giữ tối đa 1.000 request gần nhất trong 24 giờ. Bộ nhớ request bắt đầu lại khi khởi động lại HeraMind. Monitor hiển thị loại header xác thực và mã tác vụ. Giá trị khóa, tên tệp, PDF và văn bản trích xuất không có trong dữ liệu monitor.

Tab **Tác vụ OCR** hiển thị tối đa 50 tác vụ gần nhất còn trong kho OCR. Có thể xem trạng thái, số trang, thời gian từ lúc gửi đến khi kết thúc và mã lỗi. Endpoint `GET /api/ocr/monitor` yêu cầu phiên đăng nhập của quản trị viên.

## Cài phụ thuộc trên Ubuntu

```bash
sudo apt-get install poppler-utils tesseract-ocr tesseract-ocr-vie tesseract-ocr-eng
```

HeraMind tìm `pdfinfo`, `pdftoppm` và `tesseract` trong `PATH`. Nếu dùng bộ cài riêng, đặt `HERAMIND_OCR_BIN_DIR` trỏ tới thư mục chứa ba lệnh. Khởi động lại HeraMind sau khi cài hoặc thay đổi bộ ngôn ngữ.

## Kết nối runtime PaddleOCR-VL

Extension `paddle-ocr-vl` 2.14.0 gọi dịch vụ HTTP PaddleOCR-VL 1.6. Trong **Tiện ích mở rộng → PaddleOCR-VL → Cấu hình**, đặt `endpoint` tới backend suy luận. Mặc định là `http://127.0.0.1:8000`. Từ phần tổng quan của extension, chọn **Mở OCR PDF**. Với endpoint khác máy, ảnh từng trang được chuyển tới máy suy luận đã cấu hình. [Tài liệu extension 2.14.0](https://raw.githubusercontent.com/camthink-ai/NeoMind-Extensions/v2.14.0/extensions/paddle-ocr-vl/README.md)

Để chạy backend trên cùng máy, tạo môi trường Python dành riêng và cài PaddlePaddle phù hợp với GPU hoặc CPU theo hướng dẫn của PaddleOCR. Cài thêm `paddleocr[doc-parser]`, `fastapi` và `uvicorn`. Lưu `server/server.py` từ extension 2.14.0 vào thư mục runtime dưới tên `upstream_server.py` và sao chép `scripts/paddle-ocr-vl-service.py` vào cùng thư mục. [Hướng dẫn cài PaddleOCR-VL](https://github.com/PaddlePaddle/PaddleOCR/blob/main/docs/version3.x/pipeline_usage/PaddleOCR-VL.en.md#1-environment-preparation)

```bash
PADDLE_DEVICE=gpu:0 PADDLE_VL_WARMUP=0 .venv/bin/python paddle-ocr-vl-service.py
curl -fsS http://127.0.0.1:8000/health
```

Trình khởi chạy gắn cổng nội bộ `127.0.0.1:8000` và hỗ trợ cả GET lẫn POST cho kiểm tra trạng thái. Với `PADDLE_VL_WARMUP=0`, mô hình được nạp khi nhận yêu cầu OCR đầu tiên nên tác vụ đầu có thể mất thêm thời gian. Dashboard PDF trả văn bản từng trang từ lệnh `recognize` của extension. Widget ảnh gốc của extension có thêm chế độ đọc bảng và trường thông tin.
