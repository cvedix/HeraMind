import { describe, expect, it } from 'vitest'
import { toEventListItems } from '../EventList'

describe('toEventListItems', () => {
  it('joins a HERACAM vehicle event with its crop and dynamic attributes', () => {
    const payload = [
      {
        $id: 'event-line-crossing',
        event_id: 'event-1',
        ref_tracking_id: 'track-1',
        object_class: 'Vehicle',
        crossing_direction: 'up',
        tripwire_name: 'Line crossing 1',
        system_datetime: '2026-07-27T14:10:53Z',
      },
      {
        $id: 'attribute',
        ref_tracking_id: 'track-1',
        name: 'vehicle_class',
        value: 'car',
      },
      {
        $id: 'attribute',
        ref_tracking_id: 'track-1',
        name: 'vehicle_color',
        value: 'black',
      },
      {
        $id: 'attribute',
        ref_tracking_id: 'track-1',
        name: 'brand',
        value: 'Toyota',
      },
      {
        $id: 'crop',
        ref_event_id: 'event-1',
        ref_tracking_id: 'track-1',
        confidence: 0.9707,
        image: 'data:image/jpeg;base64,/9j/AA==',
      },
    ]

    const events = toEventListItems([{
      timestamp: 1785161453,
      value: JSON.stringify(payload),
    }])

    expect(events).toHaveLength(1)
    expect(events[0]).toMatchObject({
      id: 'event-1',
      eventType: 'event-line-crossing',
      objectClass: 'car',
      color: 'black',
      direction: 'up',
      confidence: 0.9707,
      tripwire: 'Line crossing 1',
      attributes: {
        vehicle_class: 'car',
        vehicle_color: 'black',
        brand: 'Toyota',
      },
    })
    expect(events[0].image).toBe('data:image/jpeg;base64,/9j/AA==')
  })

  it('keeps attributes separated when one MQTT message contains multiple vehicles', () => {
    const payload = [
      { $id: 'event-line-crossing', event_id: 'a', ref_tracking_id: 'ta', object_class: 'Vehicle' },
      { $id: 'event-line-crossing', event_id: 'b', ref_tracking_id: 'tb', object_class: 'Vehicle' },
      { $id: 'attribute', ref_tracking_id: 'ta', name: 'vehicle_color', value: 'red' },
      { $id: 'attribute', ref_tracking_id: 'tb', name: 'vehicle_color', value: 'white' },
    ]

    const events = toEventListItems([{ timestamp: 100, value: payload }])

    expect(events).toHaveLength(2)
    expect(events.find((event) => event.id === 'a')?.color).toBe('red')
    expect(events.find((event) => event.id === 'b')?.color).toBe('white')
  })

  it('accepts a pre-normalized event object', () => {
    const events = toEventListItems({
      timestamp: 1785161453,
      value: JSON.stringify({
        event_id: 'normalized-1',
        event_type: 'vehicle_event',
        vehicle_class: 'truck',
        color: 'blue',
        direction: 'down',
        attributes: { lane: 2 },
      }),
    })

    expect(events).toHaveLength(1)
    expect(events[0]).toMatchObject({
      id: 'normalized-1',
      objectClass: 'truck',
      color: 'blue',
      direction: 'down',
      attributes: { lane: 2 },
    })
  })
})
