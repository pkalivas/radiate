from __future__ import annotations

import pytest

import radiate as rd


@pytest.mark.integration
def test_handler_integration(simple_float_engine, random_seed):
    change = False

    @rd.on_epoch
    def test_handler(event: rd.EngineEvent):
        assert event.event_type == rd.EventType.EPOCH_COMPLETE
        assert event.index >= 0
        nonlocal change
        change = True

    result = (
        simple_float_engine.subscribe(test_handler).limit(rd.Limit.generations(5)).run()
    )

    assert result.index() == 5
    assert change is True


@pytest.mark.integration
def test_handler_integration_with_multiple_handlers(simple_float_engine, random_seed):
    change1 = False
    change2 = False

    @rd.on_improvement
    def test_handler1(event: rd.EngineEvent):
        assert event.event_type == rd.EventType.ENGINE_IMPROVEMENT
        assert event.index >= 0
        print(f"Improvement event at index {event}")
        nonlocal change1
        change1 = True

    @rd.on_epoch
    def test_handler2(event: rd.EngineEvent):
        assert event.event_type == rd.EventType.EPOCH_COMPLETE
        assert event.index >= 0
        nonlocal change2
        change2 = True

    result = (
        simple_float_engine.subscribe(test_handler1)
        .subscribe(test_handler2)
        .limit(rd.Limit.generations(50))
        .run()
    )
    assert result.index() == 50
    assert change1 is True
    assert change2 is True


@pytest.mark.integration
def test_receives_all_events(simple_float_engine, random_seed):
    events = {}

    @rd.on_event
    def test_handler(event: rd.EngineEvent):
        if event.event_type not in events:
            events[event.event_type] = []
        events[event.event_type].append(event)

    result = (
        simple_float_engine.subscribe(test_handler)
        .limit(rd.Limit.generations(50))
        .run()
    )

    for key, value in events.items():
        if key == rd.EventType.EPOCH_START:
            assert len(value) == result.index()
        elif key == rd.EventType.ENGINE_IMPROVEMENT:
            assert len(value) >= 1
        elif key == rd.EventType.EPOCH_COMPLETE:
            assert len(value) == result.index()
        elif key == rd.EventType.LIMIT_TRIGGERED:
            assert len(value) == 1
        elif key == rd.EventType.STOP:
            assert len(value) == 1
