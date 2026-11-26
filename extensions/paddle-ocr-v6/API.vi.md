# REST API · PaddleOCR v6

Extension ID: `paddle-ocr-v6`, phiên bản khảo sát `2.14.0`.
Thực hiện [chuẩn bị key và hàm hm_command](../../docs/extension-rest-api.vi.md)
trước khi chạy ví dụ. Endpoint:
`POST /api/extensions/paddle-ocr-v6/command`.

## Command

| Command | args | Chức năng |
| --- | --- | --- |
| `health` | `{}` | Xem trạng thái model, tier và lỗi tải |
| `switch_tier` | `tier`: string, một trong tiny/small/medium/auto | Đổi tier model của extension |
| `recognize` | Một trong `image_base64`, `image` hoặc `image_url`: string | Nhận dạng chữ trong một ảnh |

`image` là alias của image_base64. Schema hiện khai báo required rỗng cho
recognize, nhưng để OCR cần truyền dữ liệu ảnh. Nên truyền một nguồn ảnh cho
mỗi request; gửi base64 giúp tránh phụ thuộc việc server truy cập URL ngoài.

## Kiểm tra và chọn model

```bash
hm_command paddle-ocr-v6 <<'JSON'
{"command":"health","args":{}}
JSON
```

Đọc `data.loaded`, `data.tier`, `data.configured_tier`, `data.models_dir` và
`data.load_error`. Lúc khảo sát, loaded=false và load_error=null: command
health hoạt động nhưng chưa xác nhận inference OCR sẵn sàng. Model có thể được
nạp khi cần; không kết luận thiếu model chỉ từ loaded=false.

Đổi tier nếu cần; thao tác ảnh hưởng extension đang dùng chung, không chỉ
request của một client:

```bash
hm_command paddle-ocr-v6 <<'JSON'
{"command":"switch_tier","args":{"tier":"tiny"}}
JSON
```

Manifest mô tả tiny được đóng kèm, small/medium tải khi cần. Kiểm tra model và
kết nối mạng thực tế của server. Lần nạp/tải đầu có thể lâu hơn timeout ví dụ;
kiểm tra health sau khi lỗi, tránh gửi liên tục nhiều lệnh switch_tier.

## OCR ảnh từ máy client

Đặt HERAMIND_IMAGE theo hướng dẫn chung. Ví dụ gửi base64 thuần, giống widget
PaddleOCR hiện tại:

```bash
python3 - <<'PY' | hm_command paddle-ocr-v6
import base64, json, os
from pathlib import Path
image = base64.b64encode(Path(os.environ['HERAMIND_IMAGE']).read_bytes()).decode()
print(json.dumps({'command': 'recognize', 'args': {'image_base64': image}}))
PY
```

Đọc `data.full_text` và `data.text_blocks`. Theo frontend của gói đang cài,
mỗi text block có `text`, `confidence` và `bbox` gồm x/y/width/height chuẩn hóa
theo kích thước ảnh. Nhân x/width với chiều rộng ảnh, y/height với chiều cao
ảnh để vẽ khung pixel. Không dùng bbox dạng pixel của engine-bridge cho OCR.
Nội dung này được đối chiếu frontend, chưa phải kết quả OCR mới được chạy khi
viết tài liệu.

## OCR qua URL

Chỉ dùng URL mà server HeraMind có quyền và khả năng truy cập; thay URL minh
họa trước khi gửi:

```bash
hm_command paddle-ocr-v6 <<'JSON'
{"command":"recognize","args":{"image_url":"https://example.com/document.jpg"}}
JSON
```

Nếu client chọn ảnh trên máy cá nhân, dùng base64, không gửi đường dẫn cục bộ
vào image_url. Tổng JSON args vẫn tối đa 1 MiB. Không cần tạo device hoặc
camera binding để gọi OCR một ảnh.
