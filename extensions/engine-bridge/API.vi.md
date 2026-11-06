# REST API · Engine Bridge

Extension ID: `engine-bridge`, phiên bản khảo sát `0.1.0`.
Thực hiện [chuẩn bị key và hàm hm_command](../../docs/extension-rest-api.vi.md)
trong terminal trước khi chạy ví dụ. Endpoint:
`POST /api/extensions/engine-bridge/command`.

## Command

| Command | args | Kết quả trong data của REST command |
| --- | --- | --- |
| `health` | `{}` | `request_id`, `data` chứa backend/model/task |
| `get_status` | `{}` | Config, last_error, processed, failed, dropped, publish_failed |
| `get_latest` | `device_id`: string | `latest`: kết quả gần nhất hoặc null |
| `process` | `device_id`, `task`, `image_base64`: string; tùy chọn `options`: object, `source_timestamp`: Unix ms | Request ID, device ID, task, thời gian và result |
| `face_register` | `name`, `image_base64`: string | `request_id`, `data` của thao tác đăng ký |
| `face_delete` | `face_id`: integer | `request_id`, `data` của thao tác xóa |

`options` và `source_timestamp` được runtime hỗ trợ dù chưa có trong schema
command hiện tại. `device_id` trong process là nhãn nguồn, không tự tạo device.

## Kiểm tra model và xử lý ảnh

```bash
hm_command engine-bridge <<'JSON'
{"command":"health","args":{}}
JSON
```

Đọc `data.data.backend`, `data.data.seetaface_compiled` và
`data.data.age_estimation.model_available`. Máy hiện tại dùng CPU,
`qnn_available=false`; yêu cầu backend qnn bị từ chối.

Đặt `HERAMIND_IMAGE` theo hướng dẫn chung, rồi chạy ảnh qua SeetaFace:

```bash
python3 - <<'PY' | hm_command engine-bridge
import base64, json, mimetypes, os
from pathlib import Path
p = Path(os.environ['HERAMIND_IMAGE'])
mime = mimetypes.guess_type(p.name)[0] or 'application/octet-stream'
image = 'data:' + mime + ';base64,' + base64.b64encode(p.read_bytes()).decode()
print(json.dumps({'command': 'process', 'args': {
    'device_id': 'external-camera-01', 'task': 'face.recognize',
    'image_base64': image, 'options': {'backend': 'cpu', 'estimate_age': True}
}}))
PY
```

Kết quả nằm ở `data.result.faces`. Mỗi face có bbox `[x,y,width,height]`
theo pixel, detection score, keypoints và thông tin khớp danh tính nếu có.
Khi bật tuổi, có `age_estimate` và `age_estimation_status`; không có mặt thì
faces rỗng. Tuổi là ước lượng; không bật `estimate_age` thì không chạy model tuổi.

Các task khác dùng cùng command process:

| task | options |
| --- | --- |
| `image.inspect` | `{"backend":"cpu"}`: xem kích thước/pixel format/thống kê ảnh |
| `image.preprocess` | `{"backend":"cpu","width":640,"height":360,"format":"bgr"}`: resize và trả JPEG |
| `face.recognize` | `{"backend":"cpu","estimate_age":true}`: phát hiện/đối chiếu và tùy chọn tuổi |

Ảnh JPEG đầu ra của preprocess được trả cho caller trực tiếp; event/cache
không giữ trường ảnh lớn đó. Không có model suy đoán giới tính hoặc cảm xúc.

## Đăng ký và xóa

Đăng ký ảnh có một khuôn mặt rõ; thao tác này ghi vào DB SeetaFace của bridge:

```bash
python3 - <<'PY' | hm_command engine-bridge
import base64, json, os
from pathlib import Path
image = base64.b64encode(Path(os.environ['HERAMIND_IMAGE']).read_bytes()).decode()
print(json.dumps({'command': 'face_register', 'args': {
    'name': 'Nhan-do-nguoi-dung-dat', 'image_base64': image
}}))
PY
```

Giữ face ID từ kết quả đăng ký. Dùng ID integer thực tế để xóa, ví dụ `123`
chỉ là ID minh họa và phải thay trước khi gửi:

```bash
hm_command engine-bridge <<'JSON'
{"command":"face_delete","args":{"face_id":123}}
JSON
```

DB này riêng với extension `face-recognition`; đăng ký ở một extension không
tự đăng ký ở extension kia.

## Lấy kết quả và chạy qua telemetry

```bash
hm_command engine-bridge <<'JSON'
{"command":"get_latest","args":{"device_id":"external-camera-01"}}
JSON

hm_command engine-bridge <<'JSON'
{"command":"get_status","args":{}}
JSON
```

`data.latest.result` là kết quả xử lý; cache chỉ giữ bản mới nhất theo nguồn,
không phải lịch sử request. Worker xử lý từng yêu cầu, có nhánh từ chối khi bận.
Giới hạn REST args 1 MiB vẫn áp dụng dù worker chấp nhận payload lớn hơn.

Để chạy tự động từ metric ảnh, tạo thiết bị và cấu hình binding trước; xem
[cấu hình bindings](README.vi.md#nhận-ảnh-tự-động-từ-thiết-bị). `PUT config`
thay toàn bộ danh sách binding nên cần giữ các binding đang dùng. Sau đó gọi
`POST /api/devices/{device_id}/metrics` với `metric: image`, `value` là data URI.
Kết quả phát Custom event `engine.result` và cập nhật get_latest. Không có
WebSocket streaming riêng cho engine-bridge hiện tại.
