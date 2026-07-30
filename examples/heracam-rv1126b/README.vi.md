# Demo HERACAM RV1126B với MQTT topic Demo

Ví dụ này mô phỏng thiết bị **HERACAM RV1126B** trong HeraMind và ánh xạ dữ liệu
từ:

```text
MQTT 192.168.1.239:1883/Demo
  -> HERACAM RV1126B Demo
  -> transform vehicle event / crop / attribute
  -> dashboard báo cáo phương tiện
```

## Payload đã quan sát

Topic `Demo` đang gửi JSON array. Trong mẫu 30 giây, HeraMind bắt được:

| Loại object | Ý nghĩa |
|---|---|
| `event-line-crossing` | Sự kiện phương tiện đi qua line/tripwire |
| `crop` | Ảnh crop JPEG base64 của event |
| `attribute` | Thuộc tính `vehicle_class`, `vehicle_color` |

Các trường quan trọng:

| Trường | Cách dùng |
|---|---|
| `object_class` | Xác định object là `Vehicle` |
| `crossing_direction` | Hướng qua line, ví dụ `up` |
| `tripwire_name` | Tên line, ví dụ `Line crossing 1` |
| `event_id` / `ref_event_id` | Ghép event với crop tương ứng |
| `image` | Ảnh crop base64, được chuẩn hóa thành `data:image/jpeg;base64,...` |
| `name/value` | Attribute động như `vehicle_class`, `vehicle_color` |

## Dashboard

Script tạo dashboard **HERACAM RV1126B - Báo cáo phương tiện** gồm:

- Tổng phương tiện trong 24 giờ, đếm từ metric `heracam_vehicle.vehicle_seen`.
- Tổng MQTT message trong 24 giờ, đếm từ `_raw`.
- Biểu đồ số lượng phương tiện theo thời gian trong 6 giờ.
- Ảnh event mới nhất từ crop.
- Lịch sử ảnh event trong 48 giờ.
- Danh sách phương tiện trong 48 giờ, có tìm kiếm, lọc theo loại phương tiện
  và phân trang.
- Chi tiết từng event gồm ảnh crop, event ID, loại/màu/hướng phương tiện,
  độ tin cậy và toàn bộ attribute động đi kèm.
- Các thẻ thông tin: màu sắc, loại phương tiện, hướng di chuyển, độ tin cậy,
  thời gian event.

## Cài đặt

HeraMind phải đang chạy. Tạo API key trong HeraMind rồi chạy:

```bash
export HERAMIND_API_KEY='your-api-key'
./examples/heracam-rv1126b/setup.sh
```

Hoặc dùng JWT:

```bash
export HERAMIND_TOKEN='your-jwt-token'
./examples/heracam-rv1126b/setup.sh
```

Script có tính idempotent: chạy lại sẽ cập nhật device, broker, transform và
dashboard hiện có thay vì tạo bản sao.

## Kiểm thử payload mẫu

```bash
mosquitto_pub \
  -h 192.168.1.239 \
  -p 1883 \
  -t Demo \
  -f examples/heracam-rv1126b/sample-event.json
```

## Biến cấu hình

```bash
HERACAM_MQTT_HOST=192.168.1.239
HERACAM_MQTT_PORT=1883
HERACAM_MQTT_TOPIC=Demo
HERACAM_DEVICE_ID=HERACAM-RV1126B-DEMO
HERACAM_DEVICE_NAME='HERACAM RV1126B Demo'
HERACAM_MQTT_USERNAME=''
HERACAM_MQTT_PASSWORD=''
HERAMIND_API_BASE=http://127.0.0.1:9375/api
```
