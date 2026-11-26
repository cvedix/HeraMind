# Gọi extension HeraMind qua REST API

Tài liệu dành cho ứng dụng tích hợp bên ngoài. Đối chiếu ngày 06/11/2026 với
command descriptor của sáu extension đang cài trên thiết bị và mã nguồn cục bộ
khi có. Phiên bản khác có thể thay đổi tham số; luôn kiểm tra API commands.

## Hướng dẫn từng extension

| Extension | Phiên bản khảo sát | Tài liệu và chức năng |
| --- | --- | --- |
| `engine-bridge` | 0.1.0 | [Xử lý ảnh, SeetaFace, tuổi ước lượng](../extensions/engine-bridge/API.vi.md) |
| `paddle-ocr-v6` | 2.14.0 | [Nhận dạng chữ trong ảnh](../extensions/paddle-ocr-v6/API.vi.md) |
| `face-recognition` | 2.6.0 | [Binding camera, đăng ký và đối chiếu khuôn mặt](../extensions/face-recognition/API.vi.md) |
| `onvif-bridge` | 2.14.0 | [Khám phá camera ONVIF, URI và PTZ](../extensions/onvif-bridge/API.vi.md) |
| `clothing-colors` | 0.1.0 | [Phân tích màu áo/quần](../extensions/clothing-colors/API.vi.md) |
| `yolo-video` | 2.0.0 | [Tạo luồng video và phát hiện đối tượng](../extensions/yolo-video/API.vi.md) |

Một số thư mục `extensions/` chỉ lưu hướng dẫn API; package thực tế nằm trong
thư mục dữ liệu của server. Không cần cài lại hoặc restart để dùng tài liệu này.

## 1. Chuẩn bị địa chỉ và API key

Ví dụ Bash bên dưới cần `curl` và Python 3. Thực hiện trong cùng terminal để
các hàm dùng được ở những trang hướng dẫn riêng:

```bash
export HERAMIND_URL='http://192.168.1.110:9375'
read -r -s -p 'API key HeraMind: ' HERAMIND_API_KEY
printf '\n'

hm_post() {
  curl --silent --show-error --fail-with-body \
    --connect-timeout 5 --max-time 90 \
    -H "Authorization: Bearer ${HERAMIND_API_KEY}" \
    -H 'Content-Type: application/json' \
    --data-binary @- "${HERAMIND_URL%/}$1"
}

hm_command() {
  hm_post "/api/extensions/$1/command"
}
```

Nhập key khi terminal hỏi. Trên thiết bị hiện tại, có thể lấy key sẵn có từ
`.local/usb-face-feed/token`. Nếu chạy ở thư mục gốc repo, thay dòng `read`
bằng `HERAMIND_API_KEY="$(cat .local/usb-face-feed/token)"`.
Máy khác phải dùng địa chỉ server truy cập được từ máy đó; `127.0.0.1` luôn là
chính máy đang chạy client. Có thể thay HTTP bằng URL HTTPS tại điểm vào đã cấu hình.

Các lệnh thay đổi camera, binding, cấu hình hoặc dữ liệu đăng ký chỉ là ví dụ
để người tích hợp chủ động gọi. Quá trình viết tài liệu không chạy các lệnh đó.
API key hiện chưa có scope theo extension/command; giữ key trong ứng dụng server
được tin cậy, không nhúng vào frontend công khai.

## 2. Khám phá command trước khi gọi

```bash
curl --silent --show-error --fail-with-body \
  "${HERAMIND_URL%/}/api/extensions"

curl --silent --show-error --fail-with-body \
  "${HERAMIND_URL%/}/api/extensions/engine-bridge/commands"
```

Mỗi command có `id`, `input_schema.properties` và `input_schema.required`.
Schema hiện có một số chỗ chưa mô tả đầy đủ kiểu dữ liệu runtime; từng trang
riêng ghi rõ các khác biệt đã đối chiếu. `requires_auth=false` trong metadata
không miễn xác thực REST: POST command/invoke vẫn cần JWT hoặc API key.

## 3. Hai cách gọi tương đương

Gọi command trực tiếp:

```bash
hm_command engine-bridge <<'JSON'
{"command":"health","args":{}}
JSON
```

Hoặc dùng invoke:

```bash
hm_post /api/extensions/engine-bridge/invoke <<'JSON'
{"command":"health","params":{}}
JSON
```

`/command` dùng `args`; `/invoke` dùng `params`. Cả hai gọi cùng runtime và
chờ kết quả ngay trong HTTP request, không trả job ID.

Envelope rút gọn của `/command`:

```json
{"success":true,"data":{},"meta":{"timestamp":"...","request_id":"..."}}
```

Đây là hình dạng minh họa, không phải kết quả inference. `data` là kết quả
extension. Với `/invoke`, kết quả extension nằm trong `data.result`, cùng
`data.extension_id`, `data.command` và `data.timestamp`.
Một số extension còn có `success`/`error` bên trong kết quả: cần kiểm tra cả
HTTP status, `success` ngoài cùng và lỗi nghiệp vụ trong `data` nếu có.

Riêng engine-bridge có `request_id` của worker trong kết quả, khác
`meta.request_id` của API. `health` trả thông tin tại `data.data`, còn `process`
trả kết quả xử lý tại `data.result` khi gọi qua `/command`.

## 4. Đưa ảnh thật vào request

Đặt đường dẫn ảnh ở máy chạy client, ví dụ:

```bash
export HERAMIND_IMAGE='/duong/dan/anh.jpg'
```

Các trang riêng dùng Python đọc tệp này, mã hóa base64 rồi pipe JSON vào
`hm_command`/`hm_post`. Không cần đưa ảnh vào URL hoặc sửa JSON bằng tay.
Không gửi đường dẫn tệp client thay cho base64: server không đọc được tệp
trên máy của bạn. Hỗ trợ URL ảnh chỉ áp dụng cho command đã khai báo rõ.

`args`/`params` của command/invoke tối đa **1 MiB JSON sau serialize** ở API,
dù giới hạn HTTP toàn cục lớn hơn. Base64 tăng kích thước khoảng một phần ba;
ảnh khoảng 600 KiB thường chừa được chỗ cho JSON, nhưng phải tính cả tham số
khác. Giảm kích thước/quality ảnh nếu vượt giới hạn. Ví dụ PNG/JPEG phải có
MIME tương ứng khi dùng dạng `data:image/...;base64,...`.

## 5. Kết quả bất đồng bộ và lỗi thường gặp

| Tình huống | Cách xử lý |
| --- | --- |
| HTTP 401 | Kiểm tra key/JWT, header Bearer và server đang gọi |
| HTTP 404 | Kiểm tra extension ID và extension đã cài/nạp |
| HTTP 400, arguments too large | Giảm JSON args/params xuống dưới 1 MiB |
| HTTP 413 | Giảm body theo giới hạn HTTP của endpoint |
| HTTP 429 | Chờ theo chính sách rate limit, giảm nhịp request |
| HTTP 500 | Đọc error; lỗi input/model/bận/timeout có thể đang bị gom thành 500 |
| HTTP 200 nhưng lỗi trong data | Xử lý lỗi nghiệp vụ do extension trả về |
| Client timeout | Kiểm tra trạng thái/kết quả trước khi gửi lại lệnh làm thay đổi dữ liệu |

Đặt timeout client phù hợp; ví dụ curl dùng 90 giây. Không tự retry mù các
lệnh đăng ký/xóa, tạo luồng hoặc điều khiển camera vì chưa có hợp đồng idempotency.

Luồng binding/telemetry khác với command đồng bộ. Ghi metric xong không có
nghĩa inference đã hoàn tất; lấy kết quả theo cơ chế từng extension.
`GET /api/events/ws` và `GET /api/events/stream` cung cấp thông báo trực tiếp,
không thay kho kết quả bền vững. Cách xác thực streaming khác REST; xem
[phân tích gateway](extension-api-gateway.vi.md). Kiểm tra
`GET /api/extensions/{id}/stream/capability` trước khi dùng WebSocket extension.

## Kiểm chứng của tài liệu

Đã đối chiếu command của cả sáu extension và gọi các command chỉ đọc:
engine-bridge/health, paddle-ocr-v6/health, face-recognition/get_status,
clothing-colors/health, onvif-bridge/list_devices. Cả năm trả HTTP 200.
YOLO được đối chiếu descriptor và frontend, chưa tạo luồng để thử REST.
PaddleOCR hiện báo `loaded=false`; health thành công chưa xác nhận OCR inference.
Không kiểm thử di chuyển PTZ, đổi cấu hình, enrollment hoặc xóa dữ liệu đang dùng.
