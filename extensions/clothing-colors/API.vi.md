# REST API · Clothing Colors

Extension ID: `clothing-colors`, phiên bản khảo sát `0.1.0`.
Thực hiện [chuẩn bị key và hàm hm_command](../../docs/extension-rest-api.vi.md)
trước khi chạy ví dụ. Endpoint:
`POST /api/extensions/clothing-colors/command`.

Extension dùng SCHP ATR phân vùng quần áo rồi đo màu HSV/Lab. Có thể nhận
detection person từ YOLO hoặc tự phát hiện bằng model YOLO đóng kèm.

## Command và kiểu dữ liệu

| Command | args | Kết quả trong data |
| --- | --- | --- |
| `health` | `{}` | ready, method, providers |
| `analyze` | image_base64: string; detections, max_people, frame_id, publish_live, device_id tùy chọn theo runtime | persons, kích thước ảnh, processing_ms, timestamp, method |
| `get_latest` | device_id: string, có thể bỏ để dùng mặc định | sample gần nhất và danh sách events |

Khác biệt với descriptor: schema hiện ghi detections là string bắt buộc,
nhưng worker chấp nhận **mảng JSON hoặc chuỗi JSON chứa mảng**, và nếu bỏ hẳn
detections thì tự chạy YOLO. max_people, frame_id, publish_live, device_id
được worker hỗ trợ nhưng chưa khai báo đầy đủ trong schema.
Client sinh tự động từ schema cần xử lý khác biệt này; ví dụ dùng detection
ở dưới gửi chuỗi JSON phù hợp cả schema và worker.

## Kiểm tra và phân tích ảnh

```bash
hm_command clothing-colors <<'JSON'
{"command":"health","args":{}}
JSON
```

Máy hiện tại trả ready=true và providers gồm CPUExecutionProvider.
Đặt HERAMIND_IMAGE, rồi phân tích với YOLO tự phát hiện người:

```bash
python3 - <<'PY' | hm_command clothing-colors
import base64, json, os
from pathlib import Path
image = base64.b64encode(Path(os.environ['HERAMIND_IMAGE']).read_bytes()).decode()
print(json.dumps({'command': 'analyze', 'args': {
    'image_base64': image, 'max_people': 2, 'frame_id': 'client-frame-001'
}}))
PY
```

max_people được giới hạn 1–4, mặc định 2. `data.persons` có bbox theo pixel,
upper/lower chứa color, share, colors và có thể reason. Ảnh thiếu vùng quần áo
hoặc xám/IR có thể trả unknown; không diễn giải thành màu đã xác định.
Không có người đủ rõ thì persons rỗng. Thời gian và kết quả thuộc ảnh gửi lên.

## Dùng detection từ YOLO có sẵn

Ví dụ bbox dưới đây chỉ minh họa cấu trúc, cần thay bằng detection thật trên
**cùng ảnh** được gửi. bbox dùng pixel của ảnh gốc, không dùng tọa độ chuẩn hóa:

```bash
python3 - <<'PY' | hm_command clothing-colors
import base64, json, os
from pathlib import Path
image = base64.b64encode(Path(os.environ['HERAMIND_IMAGE']).read_bytes()).decode()
detections = [{'id': 1, 'class_id': 0, 'confidence': 0.95,
               'bbox': {'x': 50, 'y': 20, 'width': 200, 'height': 400}}]
print(json.dumps({'command': 'analyze', 'args': {
    'image_base64': image, 'detections': json.dumps(detections), 'max_people': 2
}}))
PY
```

Worker chỉ lấy class_id=0 (person), confidence >=0.4, crop tối thiểu 32×96
pixel sau khi giới hạn vào ảnh; mảng đầu vào tối đa 100 detection. Truyền
detections=[] nghĩa là không có người, không kích hoạt YOLO tự phát hiện.

## Công bố kết quả cho dashboard

Mặc định analyze trả kết quả trực tiếp. Để cập nhật snapshot/events nội bộ,
thêm các trường sau vào args của ví dụ phân tích:

```json
{"publish_live":true,"device_id":"camera-colors-01","frame_id":"client-frame-002"}
```

Đoạn JSON trên là **các trường bổ sung**, không phải toàn bộ request. Sau khi
analyze hoàn tất với publish_live=true, lấy snapshot bằng:

```bash
hm_command clothing-colors <<'JSON'
{"command":"get_latest","args":{"device_id":"camera-colors-01"}}
JSON
```

Kết quả là `data.sample` và `data.events`. Nếu chưa publish ảnh trong vòng đời
worker hiện tại, sample có thể null; events có thể được khôi phục từ tệp.
device_id dùng chữ Latin, số, gạch dưới hoặc gạch ngang, tối đa 128 ký tự;
worker giới hạn 10 nguồn live. Bỏ device_id thì dùng cam2-clothing-rtsp.
publish_live không tự tạo device HeraMind hoặc tự ghi lại mọi kết quả vào
telemetry. Events lưu tối đa 100 bản ghi/nguồn, hạn chế lặp tổ hợp màu trong
10 giây; không phải định danh hoặc tracking người.

REST args tối đa 1 MiB dù worker cho phép ảnh lớn hơn. Chưa có QNN/NPU trong
worker này. Xem thêm [luồng dashboard](../../docs/usb-clothing-dashboard.vi.md)
và [ghi chú model](../../docs/clothing-color-models.vi.md).
