# REST API · Face Recognition

Extension ID: `face-recognition`, phiên bản runtime khảo sát `2.6.0`.
Đây là extension SCRFD/ArcFace riêng, khác SeetaFace trong engine-bridge.
Thực hiện [chuẩn bị key và hàm hm_command/hm_post](../../docs/extension-rest-api.vi.md)
trước khi chạy ví dụ. Endpoint:
`POST /api/extensions/face-recognition/command`.

## Command

| Command | args | Chức năng |
| --- | --- | --- |
| `get_status` | `{}` | Trạng thái model, binding và thống kê |
| `get_bindings` | `{}` | Binding và kết quả camera gần nhất |
| `bind_device` | `device_id`, `metric_name`: string | Nhận ảnh tự động từ metric thiết bị |
| `unbind_device` | `device_id`: string | Gỡ binding |
| `toggle_binding` | `device_id`: string, `active`: boolean | Bật/tạm dừng binding |
| `register_face` | `name`, `image`: string | Đăng ký ảnh base64 và nhãn do người dùng đặt |
| `list_faces` | `{}` | Danh sách đã đăng ký |
| `delete_face` | `face_id`: string | Xóa một bản đăng ký |
| `get_config` | `{}` | Đọc cấu hình |
| `configure` | `config`: schema hiện quảng bá string chứa JSON | Áp dụng cấu hình; đối chiếu get_config và phiên bản trước khi thay |

Không có command process/recognize một ảnh trong descriptor phiên bản này.
Ảnh cần nhận diện được gửi qua metric của thiết bị đã binding. Nếu muốn gọi
xử lý ảnh trực tiếp trong một REST request, xem [engine-bridge](../engine-bridge/API.vi.md).

## Kiểm tra trước khi dùng

```bash
hm_command face-recognition <<'JSON'
{"command":"get_status","args":{}}
JSON

hm_command face-recognition <<'JSON'
{"command":"get_bindings","args":{}}
JSON
```

Status có model_loaded, total_bindings, total_inferences, total_recognized,
total_unknown, registered_faces và config. Số lượt nhận ra không phải số
người duy nhất. Một số model nạp khi cần; xem thêm
[hướng dẫn runtime/model](../../docs/face-recognition.vi.md).

## Liên kết thiết bị và gửi ảnh

Thiết bị phải tồn tại trong HeraMind, có metric ảnh phù hợp. Ví dụ dùng
`camera-api-01`; thay bằng ID thật của thiết bị của bạn trước khi gọi:

```bash
hm_command face-recognition <<'JSON'
{"command":"bind_device","args":{"device_id":"camera-api-01","metric_name":"image"}}
JSON
```

Nếu binding đã có, kiểm tra get_bindings trước để tránh ghi đè luồng đang dùng.
Sau khi đặt HERAMIND_IMAGE, gửi một ảnh vào telemetry:

```bash
python3 - <<'PY' | hm_post /api/devices/camera-api-01/metrics
import base64, json, mimetypes, os
from pathlib import Path
p = Path(os.environ['HERAMIND_IMAGE'])
mime = mimetypes.guess_type(p.name)[0] or 'application/octet-stream'
image = 'data:' + mime + ';base64,' + base64.b64encode(p.read_bytes()).decode()
print(json.dumps({'metric': 'image', 'value': image}))
PY
```

HTTP trả lời thao tác ghi metric trước khi inference hoàn tất. Gọi get_bindings
sau đó, chọn phần tử `data.bindings` có đúng device_id để lấy kết quả. Không
mặc định phần tử đầu tiên thuộc camera của bạn. Việc ghi ảnh vào metric tuân
theo chính sách lưu telemetry của thiết bị.

Tạm dừng hoặc gỡ binding khi không cần:

```bash
hm_command face-recognition <<'JSON'
{"command":"toggle_binding","args":{"device_id":"camera-api-01","active":false}}
JSON

hm_command face-recognition <<'JSON'
{"command":"unbind_device","args":{"device_id":"camera-api-01"}}
JSON
```

## Đăng ký và xóa khuôn mặt

Đăng ký dùng trường **image**, không phải image_base64:

```bash
python3 - <<'PY' | hm_command face-recognition
import base64, json, os
from pathlib import Path
image = base64.b64encode(Path(os.environ['HERAMIND_IMAGE']).read_bytes()).decode()
print(json.dumps({'command': 'register_face', 'args': {
    'name': 'Nhan-do-nguoi-dung-dat', 'image': image
}}))
PY
```

Ảnh cần một khuôn mặt rõ. Kiểm tra cả REST success và `data.success` nếu có;
frontend hiện xử lý lỗi nghiệp vụ đăng ký trong data. Xem danh sách và dùng
đúng ID string của bản đăng ký muốn xóa:

```bash
hm_command face-recognition <<'JSON'
{"command":"list_faces","args":{}}
JSON

hm_command face-recognition <<'JSON'
{"command":"delete_face","args":{"face_id":"THAY-BANG-ID-THUC"}}
JSON
```

Đọc cấu hình bằng `get_config`. Schema runtime không mô tả chi tiết nội dung
config của configure; tài liệu này không đưa mẫu sửa cấu hình chưa kiểm chứng.
Không sao chép config của engine-bridge sang extension này: model, binding và
database độc lập.
