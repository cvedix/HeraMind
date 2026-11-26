# REST API · ONVIF Bridge

Extension ID: `onvif-bridge`, phiên bản khảo sát `2.14.0`.
Thực hiện [chuẩn bị key và hàm hm_command](../../docs/extension-rest-api.vi.md)
trước khi chạy ví dụ. Endpoint:
`POST /api/extensions/onvif-bridge/command`.

Extension điều khiển camera ONVIF và lấy URI media; không phải model nhận dạng
ảnh. WS-Discovery và kết nối camera chạy từ máy HeraMind, không phải từ máy
client gọi REST. Server cần truy cập được mạng camera.

## Command và tham số

| Command | args bắt buộc | args tùy chọn / ý nghĩa |
| --- | --- | --- |
| `discover` | Không | timeout_ms: integer, mặc định 5000 |
| `list_devices` | Không | Liệt kê camera đã cấu hình |
| `add_device` | device: string chứa JSON cấu hình camera | Nội dung JSON phụ thuộc phiên bản extension |
| `remove_device` | device_id: string | Gỡ camera khỏi extension |
| `get_device` | device_id: string | Thông tin camera |
| `get_stream_uri` | device_id: string | profile_token: string; stream_type: RTP-Unicast hoặc RTP-Multicast |
| `get_snapshot` | device_id: string | profile_token: string; lấy URI snapshot |
| `ptz_move` | device_id: string | profile_token; pan/tilt/zoom: number; speed: number |
| `ptz_absolute` | device_id: string, pan/tilt: number | profile_token; zoom/speed: number |
| `ptz_stop` | device_id: string | profile_token: string |
| `ptz_home` | device_id: string | profile_token: string |
| `list_presets` | device_id: string | profile_token: string |
| `goto_preset` | device_id: string, preset_token: string | profile_token: string |
| `get_status` | device_id: string | profile_token: string; trạng thái PTZ của camera |
| `configure` | Schema chưa khai báo tham số | Cấu hình toàn extension; chưa có mẫu payload được xác minh |

Nếu bỏ profile_token, mô tả command cho biết dùng profile đầu tiên. Pan/tilt
trong [-1,1]; zoom relative trong [-1,1], zoom absolute trong [0,1]; speed
trong [0,1], mặc định 0.5. Camera/profile phải hỗ trợ chức năng PTZ tương ứng.

## Khám phá và chọn camera

```bash
hm_command onvif-bridge <<'JSON'
{"command":"discover","args":{"timeout_ms":5000}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"list_devices","args":{}}
JSON
```

Không mặc định camera được tìm thấy đã được thêm vào cấu hình. Dùng ID từ
list_devices cho các ví dụ tiếp theo, thay `CAMERA-ID` bằng ID thực tế:

```bash
hm_command onvif-bridge <<'JSON'
{"command":"get_device","args":{"device_id":"CAMERA-ID"}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"get_stream_uri","args":{"device_id":"CAMERA-ID","stream_type":"RTP-Unicast"}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"get_snapshot","args":{"device_id":"CAMERA-ID"}}
JSON
```

get_stream_uri lấy URI dùng cho RTSP; get_snapshot được mô tả là lấy URI ảnh,
không mặc định trả image_base64. URI media có thể mang thông tin truy cập;
lưu trong cấu hình riêng của ứng dụng thay vì ghi vào tài liệu công khai.

Schema quảng bá `add_device.device` là chuỗi JSON nhưng không công bố cấu
trúc bên trong. Bản cài cục bộ không có source/schema chi tiết để xác minh;
vì vậy tài liệu không tự đặt tên trường IP, endpoint hoặc credentials.
Cần lấy cấu trúc cấu hình đúng của phiên bản đang dùng trước khi gọi add_device.
Tương tự, chưa có ví dụ configure được xác minh. Các lệnh còn lại ở đây dùng
camera đã cấu hình sẵn và tham số được descriptor mô tả.

## PTZ và preset

Các lệnh sau làm camera dịch chuyển khi thiết bị hỗ trợ. Thay CAMERA-ID và
PRESET-TOKEN bằng giá trị từ camera của bạn:

```bash
hm_command onvif-bridge <<'JSON'
{"command":"get_status","args":{"device_id":"CAMERA-ID"}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"ptz_move","args":{"device_id":"CAMERA-ID","pan":0.1,"tilt":0,"zoom":0,"speed":0.3}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"ptz_stop","args":{"device_id":"CAMERA-ID"}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"ptz_absolute","args":{"device_id":"CAMERA-ID","pan":0,"tilt":0,"zoom":0,"speed":0.3}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"ptz_home","args":{"device_id":"CAMERA-ID"}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"list_presets","args":{"device_id":"CAMERA-ID"}}
JSON

hm_command onvif-bridge <<'JSON'
{"command":"goto_preset","args":{"device_id":"CAMERA-ID","preset_token":"PRESET-TOKEN"}}
JSON
```

Gỡ camera khỏi extension khi cần:

```bash
hm_command onvif-bridge <<'JSON'
{"command":"remove_device","args":{"device_id":"CAMERA-ID"}}
JSON
```

Kiểm tra cả success của REST và success/error bên trong data. Đã kiểm tra
list_devices trả HTTP 200; chưa chạy discover, thêm/xóa camera hay PTZ khi
viết tài liệu. Khả năng thiết bị thực tế cần được kiểm tra trên camera đích.
