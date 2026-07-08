//! Integration Tests for Rules Engine (v2 API)
//!
//! Tests cover:
//! - CompiledRule construction and evaluation
//! - Rule engine operations (add, remove, list)
//! - Value provider integration
//! - Condition evaluation (Comparison, Range, Logical)
//! - Action types (Notify, Execute, TriggerAgent)
//! - Event-driven triggering via on_data_update
//! - Cooldown behavior

use heramind_core::datasource::DataSourceId;
use heramind_rules::{
    ComparisonOperator, CompiledRule, ExecuteTarget, InMemoryValueProvider, LogicalOperator,
    NotifySeverity, RuleAction, RuleCondition, RuleEngine, RuleTrigger, RuleValue, ValueProvider,
};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// Helper: build a basic comparison rule
// ============================================================================

fn make_comparison_rule(
    name: &str,
    device_id: &str,
    metric: &str,
    operator: ComparisonOperator,
    threshold: f64,
) -> CompiledRule {
    let mut rule = CompiledRule::new(name);
    rule.condition = Some(RuleCondition::Comparison {
        source: DataSourceId::device(device_id, metric),
        operator,
        threshold,
        threshold_value: None,
    });
    rule.trigger = RuleTrigger::from_condition(&rule.condition);
    rule.actions = vec![RuleAction::Notify {
        message: format!("Rule {} triggered", name),
        severity: NotifySeverity::Warning,
    }];
    rule.finalize();
    rule
}

// ============================================================================
// Rule Engine CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_rule_engine_add_rule() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider);

    let rule = make_comparison_rule(
        "Test Rule",
        "sensor",
        "temperature",
        ComparisonOperator::GreaterThan,
        25.0,
    );
    let result = engine.add_rule(rule).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rule_engine_multiple_rules() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider);

    let rules = vec![
        make_comparison_rule("Rule1", "sensor", "a", ComparisonOperator::GreaterThan, 1.0),
        make_comparison_rule("Rule2", "sensor", "b", ComparisonOperator::GreaterThan, 2.0),
        make_comparison_rule("Rule3", "sensor", "c", ComparisonOperator::GreaterThan, 3.0),
    ];

    for rule in rules {
        let result = engine.add_rule(rule).await;
        assert!(result.is_ok(), "Failed to add rule: {:?}", result);
    }

    assert_eq!(engine.list_rules().await.len(), 3);
}

#[tokio::test]
async fn test_rule_engine_remove_rule() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider);

    let rule = make_comparison_rule(
        "Removable",
        "sensor",
        "x",
        ComparisonOperator::GreaterThan,
        0.0,
    );
    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    // Remove
    let result = engine.remove_rule(&rule_id).await;
    assert!(result.is_ok());
    assert_eq!(engine.list_rules().await.len(), 0);
}

#[tokio::test]
async fn test_rule_engine_enable_disable() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider);

    let rule = make_comparison_rule(
        "Toggle",
        "sensor",
        "temp",
        ComparisonOperator::GreaterThan,
        50.0,
    );
    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    // Disable
    engine.set_enabled(&rule_id, false).await.unwrap();
    let r = engine.get_rule(&rule_id).await.unwrap();
    assert!(!r.enabled);

    // Re-enable
    engine.set_enabled(&rule_id, true).await.unwrap();
    let r = engine.get_rule(&rule_id).await.unwrap();
    assert!(r.enabled);
}

// ============================================================================
// Value Provider Integration Tests
// ============================================================================

#[test]
fn test_in_memory_value_provider_set_and_get() {
    let provider = InMemoryValueProvider::new();

    provider.set_value("device:device1:temperature", 25.5);
    provider.set_value("device:device1:humidity", 60.0);

    let source_temp = DataSourceId::device("device1", "temperature");
    let source_hum = DataSourceId::device("device1", "humidity");
    let source_missing = DataSourceId::device("nonexistent", "metric");

    assert_eq!(
        provider.get_by_source(&source_temp),
        Some(RuleValue::Number(25.5))
    );
    assert_eq!(
        provider.get_by_source(&source_hum),
        Some(RuleValue::Number(60.0))
    );
    assert_eq!(provider.get_by_source(&source_missing), None);
}

#[test]
fn test_value_provider_update_overwrites() {
    let provider = InMemoryValueProvider::new();

    provider.set_value("device:sensor:counter", 1.0);
    let source = DataSourceId::device("sensor", "counter");
    assert_eq!(
        provider.get_by_source(&source),
        Some(RuleValue::Number(1.0))
    );

    provider.set_value("device:sensor:counter", 2.0);
    assert_eq!(
        provider.get_by_source(&source),
        Some(RuleValue::Number(2.0))
    );
}

// ============================================================================
// Condition Evaluation Tests (unit-level)
// ============================================================================

#[test]
fn test_comparison_operators_evaluate() {
    let cases = vec![
        (ComparisonOperator::GreaterThan, 30.0, 25.0, true),
        (ComparisonOperator::GreaterThan, 25.0, 30.0, false),
        (ComparisonOperator::LessThan, 20.0, 25.0, true),
        (ComparisonOperator::LessThan, 25.0, 20.0, false),
        (ComparisonOperator::GreaterEqual, 25.0, 25.0, true),
        (ComparisonOperator::GreaterEqual, 24.0, 25.0, false),
        (ComparisonOperator::LessEqual, 25.0, 25.0, true),
        (ComparisonOperator::LessEqual, 26.0, 25.0, false),
        (ComparisonOperator::Equal, 25.0, 25.0, true),
        (ComparisonOperator::Equal, 25.0, 26.0, false),
        (ComparisonOperator::NotEqual, 25.0, 30.0, true),
        (ComparisonOperator::NotEqual, 25.0, 25.0, false),
    ];

    for (op, left, right, expected) in cases {
        assert_eq!(op.evaluate(left, right), expected, "Failed for {:?}", op);
    }
}

#[test]
fn test_range_condition_with_provider() {
    let provider = InMemoryValueProvider::new();
    provider.set_value("device:sensor:temperature", 25.0);

    let source = DataSourceId::device("sensor", "temperature");

    // Inside range
    let inside = RuleCondition::Range {
        source: source.clone(),
        min: 20.0,
        max: 30.0,
    };
    assert!(inside.evaluate(&provider));

    // Below range
    let below = RuleCondition::Range {
        source: source.clone(),
        min: 30.0,
        max: 40.0,
    };
    assert!(!below.evaluate(&provider));

    // Above range
    let above = RuleCondition::Range {
        source: source.clone(),
        min: 10.0,
        max: 20.0,
    };
    assert!(!above.evaluate(&provider));
}

#[test]
fn test_logical_and_condition() {
    let provider = InMemoryValueProvider::new();
    provider.set_value("device:sensor:temperature", 35.0);
    provider.set_value("device:sensor:humidity", 40.0);

    // temperature > 30 AND humidity < 50 → both true
    let cond = RuleCondition::Logical {
        operator: LogicalOperator::And,
        conditions: vec![
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor", "temperature"),
                operator: ComparisonOperator::GreaterThan,
                threshold: 30.0,
                threshold_value: None,
            },
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor", "humidity"),
                operator: ComparisonOperator::LessThan,
                threshold: 50.0,
                threshold_value: None,
            },
        ],
    };
    assert!(cond.evaluate(&provider));
}

#[test]
fn test_logical_or_condition() {
    let provider = InMemoryValueProvider::new();
    provider.set_value("device:sensor:temperature", 20.0);
    provider.set_value("device:sensor:humidity", 95.0);

    // temperature > 40 (false) OR humidity > 90 (true) → true
    let cond = RuleCondition::Logical {
        operator: LogicalOperator::Or,
        conditions: vec![
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor", "temperature"),
                operator: ComparisonOperator::GreaterThan,
                threshold: 40.0,
                threshold_value: None,
            },
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor", "humidity"),
                operator: ComparisonOperator::GreaterThan,
                threshold: 90.0,
                threshold_value: None,
            },
        ],
    };
    assert!(cond.evaluate(&provider));
}

#[test]
fn test_logical_not_condition() {
    let provider = InMemoryValueProvider::new();
    provider.set_value("device:sensor:temperature", 20.0);

    // NOT (temperature > 30) → true because 20 is not > 30
    let cond = RuleCondition::Logical {
        operator: LogicalOperator::Not,
        conditions: vec![RuleCondition::Comparison {
            source: DataSourceId::device("sensor", "temperature"),
            operator: ComparisonOperator::GreaterThan,
            threshold: 30.0,
            threshold_value: None,
        }],
    };
    assert!(cond.evaluate(&provider));
}

#[test]
fn test_nested_conditions() {
    let provider = InMemoryValueProvider::new();
    provider.set_value("device:sensor:temp", 35.0);
    provider.set_value("device:sensor:humidity", 60.0);

    // ((temp > 30) OR (temp < 10)) AND (humidity > 50)
    let cond = RuleCondition::Logical {
        operator: LogicalOperator::And,
        conditions: vec![
            RuleCondition::Logical {
                operator: LogicalOperator::Or,
                conditions: vec![
                    RuleCondition::Comparison {
                        source: DataSourceId::device("sensor", "temp"),
                        operator: ComparisonOperator::GreaterThan,
                        threshold: 30.0,
                        threshold_value: None,
                    },
                    RuleCondition::Comparison {
                        source: DataSourceId::device("sensor", "temp"),
                        operator: ComparisonOperator::LessThan,
                        threshold: 10.0,
                        threshold_value: None,
                    },
                ],
            },
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor", "humidity"),
                operator: ComparisonOperator::GreaterThan,
                threshold: 50.0,
                threshold_value: None,
            },
        ],
    };
    assert!(cond.evaluate(&provider));
}

// ============================================================================
// Condition Source Extraction Tests
// ============================================================================

#[test]
fn test_condition_extract_sources_comparison() {
    let cond = RuleCondition::Comparison {
        source: DataSourceId::device("sensor1", "temperature"),
        operator: ComparisonOperator::GreaterThan,
        threshold: 30.0,
        threshold_value: None,
    };
    let sources = cond.extract_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].storage_key(), "device:sensor1:temperature");
}

#[test]
fn test_condition_extract_sources_logical() {
    let cond = RuleCondition::Logical {
        operator: LogicalOperator::And,
        conditions: vec![
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor1", "temp"),
                operator: ComparisonOperator::GreaterThan,
                threshold: 30.0,
                threshold_value: None,
            },
            RuleCondition::Comparison {
                source: DataSourceId::device("sensor2", "humidity"),
                operator: ComparisonOperator::LessThan,
                threshold: 50.0,
                threshold_value: None,
            },
        ],
    };
    let sources = cond.extract_sources();
    assert_eq!(sources.len(), 2);
}

// ============================================================================
// Event-Driven Triggering Tests (on_data_update)
// ============================================================================

#[tokio::test]
async fn test_on_data_update_triggers_rule() {
    let provider = Arc::new(InMemoryValueProvider::new());
    provider.set_value("device:sensor:temperature", 35.0);

    let engine = RuleEngine::new(provider.clone());

    let mut rule = CompiledRule::new("Device Metric");
    rule.condition = Some(RuleCondition::Comparison {
        source: DataSourceId::device("sensor", "temperature"),
        operator: ComparisonOperator::GreaterThan,
        threshold: 30.0,
        threshold_value: None,
    });
    rule.trigger = RuleTrigger::from_condition(&rule.condition);
    rule.actions = vec![RuleAction::Notify {
        message: "High temperature".into(),
        severity: NotifySeverity::Warning,
    }];
    rule.finalize();

    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    // Trigger the rule via data update
    engine
        .on_data_update(
            &DataSourceId::device("sensor", "temperature"),
            RuleValue::Number(35.0),
        )
        .await;

    let r = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(r.state.trigger_count, 1);
}

#[tokio::test]
async fn test_on_data_update_below_threshold_no_trigger() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider.clone());

    let mut rule = CompiledRule::new("High Temp");
    rule.condition = Some(RuleCondition::Comparison {
        source: DataSourceId::device("sensor", "temperature"),
        operator: ComparisonOperator::GreaterThan,
        threshold: 50.0,
        threshold_value: None,
    });
    rule.trigger = RuleTrigger::from_condition(&rule.condition);
    rule.finalize();

    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    provider.set_value("device:sensor:temperature", 25.0);
    engine
        .on_data_update(
            &DataSourceId::device("sensor", "temperature"),
            RuleValue::Number(25.0),
        )
        .await;

    let r = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(r.state.trigger_count, 0);
}

#[tokio::test]
async fn test_cooldown_prevents_retrigger() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider.clone());

    let mut rule = CompiledRule::new("High Temp");
    rule.condition = Some(RuleCondition::Comparison {
        source: DataSourceId::device("sensor", "temperature"),
        operator: ComparisonOperator::GreaterThan,
        threshold: 50.0,
        threshold_value: None,
    });
    rule.trigger = RuleTrigger::from_condition(&rule.condition);
    rule.cooldown = Duration::from_secs(60);
    rule.actions = vec![RuleAction::Notify {
        message: "Too hot".into(),
        severity: NotifySeverity::Warning,
    }];
    rule.finalize();

    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    // First trigger
    provider.set_value("device:sensor:temperature", 75.0);
    engine
        .on_data_update(
            &DataSourceId::device("sensor", "temperature"),
            RuleValue::Number(75.0),
        )
        .await;

    // Second trigger (within cooldown — should be suppressed)
    engine
        .on_data_update(
            &DataSourceId::device("sensor", "temperature"),
            RuleValue::Number(80.0),
        )
        .await;

    let r = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(r.state.trigger_count, 1);
}

// ============================================================================
// Manual Trigger Tests
// ============================================================================

#[tokio::test]
async fn test_manual_trigger_no_condition() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider.clone());

    let mut rule = CompiledRule::new("Manual Rule");
    rule.trigger = RuleTrigger::Manual;
    // No condition — always fires on manual trigger
    rule.actions = vec![RuleAction::Notify {
        message: "Manual fired".into(),
        severity: NotifySeverity::Info,
    }];
    rule.finalize();

    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    let result = engine.execute_rule(&rule_id).await;
    assert!(result.success);

    let r = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(r.state.trigger_count, 1);
}

// ============================================================================
// Action Construction Tests
// ============================================================================

#[test]
fn test_notify_action_construction() {
    let action = RuleAction::Notify {
        message: "Alert message".into(),
        severity: NotifySeverity::Critical,
    };
    match &action {
        RuleAction::Notify { message, severity } => {
            assert_eq!(message, "Alert message");
            assert_eq!(*severity, NotifySeverity::Critical);
        }
        _ => panic!("Expected Notify action"),
    }
}

#[test]
fn test_execute_action_construction() {
    let action = RuleAction::Execute {
        target: "device1".into(),
        target_type: ExecuteTarget::Device,
        command: "turn_on".into(),
        params: serde_json::json!({"mode": "auto", "speed": 100}),
    };
    match &action {
        RuleAction::Execute {
            target,
            target_type,
            command,
            params,
        } => {
            assert_eq!(target, "device1");
            assert_eq!(*target_type, ExecuteTarget::Device);
            assert_eq!(command, "turn_on");
            assert_eq!(params["mode"], "auto");
            assert_eq!(params["speed"], 100);
        }
        _ => panic!("Expected Execute action"),
    }
}

#[test]
fn test_trigger_agent_action_construction() {
    let action = RuleAction::TriggerAgent {
        agent_id: "agent-123".into(),
        input: Some("Check temperature".into()),
        data: Some(serde_json::json!({"temperature": 35.0})),
    };
    match &action {
        RuleAction::TriggerAgent {
            agent_id,
            input,
            data,
        } => {
            assert_eq!(agent_id, "agent-123");
            assert_eq!(input.as_deref(), Some("Check temperature"));
            assert!(data.is_some());
        }
        _ => panic!("Expected TriggerAgent action"),
    }
}

// ============================================================================
// Rule Metadata Tests
// ============================================================================

#[tokio::test]
async fn test_rule_with_description_and_tags() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider);

    let mut rule = CompiledRule::new("Tagged Rule");
    rule.description = Some("Monitors temperature".into());
    rule.tags = vec!["temperature".into(), "alert".into(), "critical".into()];
    rule.condition = Some(RuleCondition::Comparison {
        source: DataSourceId::device("sensor", "temp"),
        operator: ComparisonOperator::GreaterThan,
        threshold: 40.0,
        threshold_value: None,
    });
    rule.trigger = RuleTrigger::from_condition(&rule.condition);
    rule.finalize();

    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    let fetched = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(fetched.name, "Tagged Rule");
    assert_eq!(fetched.description.as_deref(), Some("Monitors temperature"));
    assert_eq!(fetched.tags.len(), 3);
    assert!(fetched.tags.contains(&"temperature".to_string()));
    assert!(fetched.enabled);
}

#[tokio::test]
async fn test_rule_with_for_duration() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider);

    let mut rule = CompiledRule::new("Persistent Alert");
    rule.condition = Some(RuleCondition::Comparison {
        source: DataSourceId::device("sensor", "temperature"),
        operator: ComparisonOperator::GreaterThan,
        threshold: 30.0,
        threshold_value: None,
    });
    rule.trigger = RuleTrigger::from_condition(&rule.condition);
    rule.for_duration = Some(Duration::from_secs(300)); // 5 minutes
    rule.finalize();

    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    let fetched = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(fetched.for_duration, Some(Duration::from_secs(300)));
}

// ============================================================================
// Subscription Index / Selective Trigger Tests
// ============================================================================

#[tokio::test]
async fn test_subscription_index_selective() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider.clone());

    // Rule 1: watches sensor1
    let rule1 = make_comparison_rule(
        "Rule 1",
        "sensor1",
        "temp",
        ComparisonOperator::GreaterThan,
        50.0,
    );
    let rule1_id = rule1.id.clone();

    // Rule 2: watches sensor2
    let rule2 = make_comparison_rule(
        "Rule 2",
        "sensor2",
        "temp",
        ComparisonOperator::GreaterThan,
        50.0,
    );
    let rule2_id = rule2.id.clone();

    engine.add_rule(rule1).await.unwrap();
    engine.add_rule(rule2).await.unwrap();

    // Update sensor1 — only rule1 should trigger
    provider.set_value("device:sensor1:temp", 75.0);
    engine
        .on_data_update(
            &DataSourceId::device("sensor1", "temp"),
            RuleValue::Number(75.0),
        )
        .await;

    let r1 = engine.get_rule(&rule1_id).await.unwrap();
    let r2 = engine.get_rule(&rule2_id).await.unwrap();
    assert_eq!(r1.state.trigger_count, 1);
    assert_eq!(r2.state.trigger_count, 0);
}

// ============================================================================
// Disabled Rule Does Not Trigger
// ============================================================================

#[tokio::test]
async fn test_disabled_rule_does_not_trigger() {
    let provider = Arc::new(InMemoryValueProvider::new());
    let engine = RuleEngine::new(provider.clone());

    let rule = make_comparison_rule(
        "Disabled",
        "sensor",
        "temp",
        ComparisonOperator::GreaterThan,
        50.0,
    );
    let rule_id = rule.id.clone();
    engine.add_rule(rule).await.unwrap();

    engine.set_enabled(&rule_id, false).await.unwrap();

    provider.set_value("device:sensor:temp", 75.0);
    engine
        .on_data_update(
            &DataSourceId::device("sensor", "temp"),
            RuleValue::Number(75.0),
        )
        .await;

    let r = engine.get_rule(&rule_id).await.unwrap();
    assert_eq!(r.state.trigger_count, 0);
}
