from __future__ import annotations

import pytest

import radiate as rd


@pytest.mark.integration
def test_handler_integration(simple_float_engine, random_seed):
    change = False

    @rd.on_epoch
    def test_handler(event: rd.EngineEvent):
        assert event.event_type() == rd.EventType.EPOCH_COMPLETE
        assert event.index() >= 0
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
        assert event.event_type() == rd.EventType.ENGINE_IMPROVEMENT
        assert event.index() >= 0
        nonlocal change1
        change1 = True

    @rd.on_epoch
    def test_handler2(event: rd.EngineEvent):
        assert event.event_type() == rd.EventType.EPOCH_COMPLETE
        assert event.index() >= 0
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
