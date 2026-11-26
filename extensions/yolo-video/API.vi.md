# REST API · YOLO Video

Extension ID: `yolo-video`, phiên bản khảo sát `2.0.0`.
Thực hiện [chuẩn bị key và hàm hm_command](../../docs/extension-rest-api.vi.md)
trước khi chạy ví dụ. Endpoint:
`POST /api/extensions/yolo-video/command`.

Extension tạo luồng video và phát hiện đối tượng. Các ví dụ dưới đây theo
descriptor đang chạy và frontend được cài; chưa tạo/dừng luồng thật để thử
REST khi viết tài liệu.

## Command

| Command | args | Chức năng |
| --- | --- | --- |
| `start_stream` | source_url: string, schema cho phép bỏ | Tạo luồng mới từ nguồn video |
| `stop_stream` | stream_id: string | Dừng luồng đã tạo |
| `get_stream_stats` | stream_id: string | Thống kê luồng |
| `get_frame` | stream_id: string | Lấy frame hiện tại dạng base64 JPEG |
| `update_stream_config` | stream_id và các trường cấu hình | Cập nhật ROI/đường đếm/capture rules |

Schema update_stream_config hiện không khai báo tham số; frontend thực tế
gửi stream_id, rois, lines và capture_rules. Chưa có schema chi tiết của các
phần tử này trong descriptor; không tự dựng cấu trúc ROI từ tọa độ bất kỳ.

## Tạo luồng và lấy frame

Thay URL ví dụ bằng nguồn server có thể truy cập. Lệnh tạo luồng mới, có thể
tốn tài nguyên xử lý trên thiết bị:

```bash
hm_command yolo-video <<'JSON'
{"command":"start_stream","args":{"source_url":"rtsp://CAMERA-HOST:554/STREAM-PATH"}}
JSON
```

Giữ stream_id do extension trả về và thay STREAM-ID trong các lệnh sau.
Descriptor không công bố output schema nên kiểm tra payload thực tế khi
tích hợp phiên bản khác. Không dùng device_id hoặc URL thay stream_id.
Nguồn camera được mở từ server, không phải webcam của trình duyệt client.

```bash
hm_command yolo-video <<'JSON'
{"command":"get_stream_stats","args":{"stream_id":"STREAM-ID"}}
JSON

hm_command yolo-video <<'JSON'
{"command":"get_frame","args":{"stream_id":"STREAM-ID"}}
JSON
```

REST command trả JSON. get_frame được mô tả là ảnh JPEG base64; đọc field ảnh
trong payload thực tế trước khi decode hoặc tạo data URI. Không giả định
endpoint command trả trực tiếp luồng MJPEG/video. Tần suất polling cần phù hợp
tải inference và băng thông của thiết bị.

## Cập nhật cấu hình và dừng

Frontend hiện dùng command update_stream_config với ba mảng. Ví dụ dưới
**xóa toàn bộ ROI, đường đếm và capture rules** của luồng được chỉ định:

```bash
hm_command yolo-video <<'JSON'
{"command":"update_stream_config","args":{"stream_id":"STREAM-ID","rois":[],"lines":[],"capture_rules":[]}}
JSON
```

Để giữ hoặc sửa cấu hình đã vẽ trên dashboard, gửi cấu trúc dữ liệu đúng của
frontend cùng phiên bản. Tài liệu này chưa xác minh cấu trúc phần tử để tạo
ROI/line/capture rule mới bằng REST.

```bash
hm_command yolo-video <<'JSON'
{"command":"stop_stream","args":{"stream_id":"STREAM-ID"}}
JSON
```

## Streaming và dashboard

```bash
curl --silent --show-error --fail-with-body \
  "${HERAMIND_URL%/}/api/extensions/yolo-video/stream/capability"
```

Kiểm tra hiện tại trả mode=push, direction=bidirectional,
supported_data_types=[image/jpeg], max_concurrent_sessions=4.
WebSocket `/api/extensions/yolo-video/stream` là transport riêng, không thể
thay bằng POST command để nhận liên tục frame. Tích hợp WebSocket cần thực
hiện xác thực và giao thức session của server.
`GET /api/extensions/yolo-video/stream/sessions` trả session_id của SDK;
không mặc định đồng nhất nó với stream_id của command REST.

Để xem video và thao tác ROI có sẵn, dùng
[dashboard YOLO](../../docs/rtsp-dashboard.vi.md). Số lượt detection hoặc số
khung hình không đồng nghĩa với số người duy nhất hoặc lượt qua đường đếm.
