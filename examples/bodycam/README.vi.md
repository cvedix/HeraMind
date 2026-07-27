# Demo BodyCam với MQTT Events

Ví dụ này tạo một luồng hoàn chỉnh trong HeraMind:

```text
MQTT 192.168.1.239:1883/events
  → BodyCam G25A06689
  → transform attribute / crop / event
  → metric có kiểu dữ liệu rõ ràng
  → dashboard realtime
```

## Dữ liệu thiết bị

| Trường | Giá trị |
|---|---|
| Thiết bị | BodyCam / Wearable Video Recorder |
| S/N | `G25A06689` |
| IP | `192.168.1.185` |
| MAC | `40-D9-5A-EA-D1-57` |
| MQTT broker | `192.168.1.239:1883` |
| Topic | `events` |
| HLS | `http://192.168.1.239:3546/hls/3e0d9c56-3f25-c54b-2933-ba0e23720611.m3u8` |

Payload thực tế trên `events` là một mảng gồm nhiều loại object. Các phần tử
`{"$id":"attribute","name":"...","value":"..."}` được transform thành metric
như `bodycam_event.smoking`, `bodycam_event.phone`, `bodycam_event.age` và
`bodycam_event.gender`. Phần tử `{"$id":"crop","image":"<JPEG base64>"}` được
chuẩn hóa thành ảnh event; transform ưu tiên crop có
`ref_event_id` khớp với `event_id`, sau đó chọn `confidence` cao nhất. Các dạng
cũ có tên `crop`, `crop_image`, `cropped_image`, `event_crop`, `image_crop`,
`snapshot` hoặc `thumbnail` vẫn được chuẩn hóa thành
`bodycam_event.crop_image`. Giá trị ảnh
có thể là URL, data URL/base64 hoặc object chứa `url/src/image/data/base64`.

`face_features` được loại khỏi transform vì đây là vector đặc trưng khuôn mặt
rất lớn, không phù hợp để hiển thị hoặc vẽ biểu đồ. Payload gốc vẫn được HeraMind
lưu trong metric `_raw` để phục vụ điều tra khi cần.

### Phân tích payload đã quan sát

- Mỗi message đại diện cho một tập thuộc tính của cùng một tracking event.
- Widget **MQTT messages · 24 giờ** đếm trực tiếp số điểm `_raw`, tương ứng
  số MQTT publish HeraMind nhận được; không nhầm với số thuộc tính trong mảng.
- Dashboard hiển thị crop mới nhất và tối đa 200 crop trong 48 giờ gần nhất.
- `ref_tracking_id` liên kết các thuộc tính trong cùng event.
- `instance_id` xác định instance của hệ thống phân tích, không phải S/N BodyCam.
- `event_timestamp_ms` là thời gian tương đối của luồng; `system_datetime` và
  `system_timestamp` là thời gian hệ thống.
- Các boolean đang được gửi dưới dạng chuỗi (`"true"`/`"false"`); transform
  chuyển chúng thành boolean thật trước khi đưa lên dashboard.
- Payload không chứa `G25A06689`, IP hoặc MAC. Vì vậy cấu hình mẫu ánh xạ toàn bộ
  topic `events` vào BodyCam này. Nếu broker gom sự kiện từ nhiều camera, cần bổ
  sung device ID vào payload hoặc tách topic, ví dụ `events/G25A06689`.

## Cài đặt

HeraMind phải đang chạy. Tạo API key trong HeraMind rồi chạy:

```bash
export HERAMIND_API_KEY='your-api-key'
./examples/bodycam/setup.sh
```

Nếu đang dùng JWT của phiên đăng nhập:

```bash
export HERAMIND_TOKEN='your-jwt-token'
./examples/bodycam/setup.sh
```

Script có tính idempotent: chạy lại sẽ cập nhật broker, device, transform và
dashboard hiện có thay vì tạo bản sao.

## Kiểm thử với payload mẫu

```bash
mosquitto_pub \
  -h 192.168.1.239 \
  -p 1883 \
  -t events \
  -f examples/bodycam/sample-event.json
```

Sau đó mở `http://localhost:9375`, chọn dashboard
**BodyCam Analytics - G25A06689**. Dashboard gồm live HLS, crop mới nhất, lịch
sử crop, bộ đếm MQTT và các thuộc tính analytics.

Nếu HLS không phát được, kiểm tra trình duyệt truy cập được URL `.m3u8` và các
segment, server HLS cho phép CORS, và dashboard không chạy HTTPS trong khi HLS
dùng HTTP (mixed content sẽ bị trình duyệt chặn).

> Broker mẫu đang dùng MQTT không TLS và không tài khoản trong mạng LAN. Khi đưa
> lên môi trường production, nên bật xác thực/TLS hoặc cô lập broker trong VLAN.

## Biến cấu hình

Có thể ghi đè các giá trị mà không sửa script:

```bash
BODYCAM_MQTT_HOST=192.168.1.239
BODYCAM_MQTT_PORT=1883
BODYCAM_MQTT_TOPIC=events
BODYCAM_HLS_URL=http://192.168.1.239:3546/hls/3e0d9c56-3f25-c54b-2933-ba0e23720611.m3u8
BODYCAM_MQTT_USERNAME=''
BODYCAM_MQTT_PASSWORD=''
BODYCAM_DEVICE_ID=G25A06689
HERAMIND_API_BASE=http://127.0.0.1:9375/api
```
