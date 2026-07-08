use super::*;

fn metric_value_to_json(
    v: &heramind_core::extension::system::ParamMetricValue,
) -> serde_json::Value {
    match v {
        heramind_core::extension::system::ParamMetricValue::Float(v) => serde_json::json!(*v),
        heramind_core::extension::system::ParamMetricValue::Integer(v) => serde_json::json!(*v),
        heramind_core::extension::system::ParamMetricValue::Boolean(v) => serde_json::json!(*v),
        heramind_core::extension::system::ParamMetricValue::String(v) => serde_json::json!(v),
        heramind_core::extension::system::ParamMetricValue::Null => serde_json::Value::Null,
        heramind_core::extension::system::ParamMetricValue::Binary(_) => {
            serde_json::json!("<binary data>")
        }
    }
}

pub(crate) fn get_time_context() -> String {
    use heramind_storage::SettingsStore;

    const SETTINGS_DB_PATH: &str = "data/settings.redb";

    // Try to load timezone from settings
    let timezone = SettingsStore::open(SETTINGS_DB_PATH)
        .ok()
        .map(|store| store.get_global_timezone())
        .unwrap_or_else(|| "Asia/Shanghai".to_string());

    let now = chrono::Utc::now();

    // Parse timezone
    let tz = timezone
        .parse::<chrono_tz::Tz>()
        .unwrap_or(chrono_tz::Tz::Asia__Shanghai);

    // Get current time in the configured timezone
    let local_now = now.with_timezone(&tz);

    // Format various time components
    let local_time = local_now.format("%Y-%m-%d %H:%M:%S").to_string();
    let date_str = local_now.format("%B %d, %Y").to_string();
    let day_of_week = local_now.format("%A").to_string(); // Monday, Tuesday, etc.

    // Get time period description - use format to get hour value
    let hour_str = local_now.format("%H").to_string();
    let hour: u32 = hour_str.parse().unwrap_or(12);
    let time_period = match hour {
        5..=11 => "Morning",
        12..=13 => "Noon",
        14..=17 => "Afternoon",
        18..=22 => "Evening",
        _ => "Night",
    };

    format!(
        "{} {} ({}, {} {})",
        local_time, timezone, date_str, day_of_week, time_period
    )
}

impl AgentExecutor {
    pub(crate) async fn collect_data(&self, agent: &AiAgent) -> AgentResult<Vec<DataCollected>> {
        let timestamp = chrono::Utc::now().timestamp();

        // DEBUG: Log data collection start
        tracing::info!(
            agent_id = %agent.id,
            agent_name = %agent.name,
            total_resources = agent.resources.len(),
            has_time_series_storage = self.time_series_storage.is_some(),
            "[COLLECT] Starting data collection"
        );

        // Split resources by type for parallel processing
        let metric_resources: Vec<_> = agent
            .resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Metric)
            .cloned()
            .collect();

        // Extract device IDs and their bound metrics from Metric resources
        // Format: "device_id:metric_name"
        let mut device_bound_metrics: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for resource in &metric_resources {
            let parts: Vec<&str> = resource.resource_id.split(':').collect();
            if parts.len() == 2 {
                let (device_id, metric_name) = (parts[0], parts[1]);
                device_bound_metrics
                    .entry(device_id.to_string())
                    .or_default()
                    .push(metric_name.to_string());
            }
        }

        let device_resources: Vec<_> = agent
            .resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Device)
            .map(|r| r.resource_id.clone())
            .collect();

        let extension_metric_resources: Vec<_> = agent
            .resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::ExtensionMetric)
            .cloned()
            .collect();

        tracing::debug!(
            agent_id = %agent.id,
            metric_count = metric_resources.len(),
            device_count = device_resources.len(),
            extension_metric_count = extension_metric_resources.len(),
            "[COLLECT] Resource breakdown"
        );

        // Check if time_series_storage is available
        if self.time_series_storage.is_none() {
            tracing::warn!(
                agent_id = %agent.id,
                "[COLLECT] Time series storage is NOT available - data collection will fail!"
            );
        }

        // Collect metric data in parallel
        let metric_data = self
            .collect_metric_data_parallel(agent, metric_resources, timestamp)
            .await?;
        tracing::debug!(
            agent_id = %agent.id,
            metric_data_count = metric_data.len(),
            "[COLLECT] Metric data collected"
        );

        // Collect device data in parallel
        let device_data = self
            .collect_device_data_parallel(agent, device_resources, device_bound_metrics, timestamp)
            .await?;
        tracing::debug!(
            agent_id = %agent.id,
            device_data_count = device_data.len(),
            "[COLLECT] Device data collected"
        );

        // Collect extension metric data in parallel
        let extension_data = self
            .collect_extension_metric_data_parallel(agent, extension_metric_resources, timestamp)
            .await?;
        tracing::debug!(
            agent_id = %agent.id,
            extension_data_count = extension_data.len(),
            "[COLLECT] Extension metric data collected"
        );

        // Combine all data, deduplicating across channels.
        //
        // The metric channel and device channel can both collect the same
        // (device_id, metric_name) when an agent has overlapping Metric +
        // Device resources. Metric-channel entries take priority (they carry
        // richer config: time_range, history, trend). We dedup by (source,
        // data_type) which is identical across both channels for the same
        // device+metric pair.
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let mut data: Vec<DataCollected> = Vec::new();
        for item in metric_data
            .into_iter()
            .chain(device_data.into_iter())
            .chain(extension_data.into_iter())
        {
            let key = (item.source.clone(), item.data_type.clone());
            if seen.insert(key) {
                data.push(item);
            }
        }
        let deduped_count = data.len();

        // Add condensed memory context
        let memory_data = self.collect_memory_summary(agent, timestamp)?;
        data.extend(memory_data);

        // Log summary of collected data
        tracing::info!(
            agent_id = %agent.id,
            total_collected = data.len(),
            deduped_after = deduped_count,
            data_sources = ?data.iter().map(|d| format!("{}:{}", d.source, d.data_type)).collect::<Vec<_>>(),
            "[COLLECT] Data collection summary"
        );

        // If no data collected, add a placeholder
        if data.is_empty() {
            tracing::warn!(
                agent_id = %agent.id,
                "[COLLECT] NO DATA COLLECTED - adding placeholder"
            );
            data.push(DataCollected {
                source: "system".to_string(),
                data_type: "info".to_string(),
                values: serde_json::json!({
                    "message": "No pre-collected data available. Use available tools to query device data as needed, or analyze based on user instructions and historical patterns."
                }),
                timestamp,
            });
        }

        Ok(data)
    }

    pub(crate) async fn collect_metric_data_parallel(
        &self,
        _agent: &AiAgent, // Reserved for future use
        resources: Vec<AgentResource>,
        timestamp: i64,
    ) -> AgentResult<Vec<DataCollected>> {
        // If no resources, return empty data without requiring storage
        if resources.is_empty() {
            tracing::debug!("No metric resources to collect, returning empty data");
            return Ok(vec![]);
        }

        let storage = self
            .time_series_storage
            .clone()
            .ok_or(HeraMindError::validation(
                "Time series storage not available".to_string(),
            ))?;

        // Create parallel futures for each metric resource
        let collect_futures: Vec<_> = resources
            .into_iter()
            .filter_map(|resource| {
                // Parse device_id and metric from resource_id (format: "device_id:metric_name")
                let parts: Vec<&str> = resource.resource_id.split(':').collect();
                if parts.len() != 2 {
                    return None;
                }
                let (device_id, metric_name) = (parts[0], parts[1]);

                // Extract config
                let time_range_minutes = resource
                    .config
                    .get("data_collection")
                    .and_then(|dc| dc.get("time_range_minutes"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60);

                let include_history = resource
                    .config
                    .get("data_collection")
                    .and_then(|dc| dc.get("include_history"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let max_points = resource
                    .config
                    .get("data_collection")
                    .and_then(|dc| dc.get("max_points"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as usize;

                let include_trend = resource
                    .config
                    .get("data_collection")
                    .and_then(|dc| dc.get("include_trend"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let include_baseline = resource
                    .config
                    .get("data_collection")
                    .and_then(|dc| dc.get("include_baseline"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Clone necessary data for the async block
                let resource_id = resource.resource_id.clone();
                let storage_clone = storage.clone();
                let metric_name = metric_name.to_string();
                let device_id = device_id.to_string();

                Some(async move {
                    Self::collect_single_metric(
                        storage_clone,
                        &device_id,
                        &metric_name,
                        resource_id,
                        time_range_minutes,
                        include_history,
                        max_points,
                        include_trend,
                        include_baseline,
                        timestamp,
                    )
                    .await
                })
            })
            .collect();

        // Execute all queries in parallel with timeout
        // Each query gets a maximum of 10 seconds to complete
        const QUERY_TIMEOUT_SECS: u64 = 10;

        let timeout_futures: Vec<_> = collect_futures
            .into_iter()
            .map(|fut| async move {
                match tokio::time::timeout(std::time::Duration::from_secs(QUERY_TIMEOUT_SECS), fut)
                    .await
                {
                    Ok(result) => result,
                    Err(_) => {
                        tracing::warn!(
                            "Data collection query timed out after {}s",
                            QUERY_TIMEOUT_SECS
                        );
                        Err(HeraMindError::Llm(format!(
                            "Query timeout after {}s",
                            QUERY_TIMEOUT_SECS
                        )))
                    }
                }
            })
            .collect();

        let results = join_all(timeout_futures).await;

        // Filter out errors and collect successful results
        let collected: Vec<_> = results
            .into_iter()
            .filter_map(|r| {
                if let Err(ref e) = r {
                    tracing::warn!(error = %e, "Metric data collection failed");
                }
                r.ok()
            })
            .flatten()
            .collect();
        Ok(collected)
    }

    pub(crate) async fn collect_single_metric(
        storage: Arc<heramind_storage::TimeSeriesStore>,
        device_id: &str,
        metric_name: &str,
        resource_id: String,
        time_range_minutes: u64,
        include_history: bool,
        max_points: usize,
        _include_trend: bool,    // Reserved for future use
        _include_baseline: bool, // Reserved for future use
        timestamp: i64,
    ) -> AgentResult<Option<DataCollected>> {
        let end_time = chrono::Utc::now().timestamp();
        let start_time = end_time - ((time_range_minutes * 60) as i64);

        tracing::debug!(
            device_id = %device_id,
            metric_name = %metric_name,
            time_range_minutes = %time_range_minutes,
            start_time = %start_time,
            end_time = %end_time,
            "[COLLECT] Querying metric"
        );

        let result = storage
            .query_range(device_id, metric_name, start_time, end_time, None)
            .await
            .map_err(|e| HeraMindError::Storage(format!("Query failed: {}", e)))?;

        if result.points.is_empty() {
            tracing::debug!(
                device_id = %device_id,
                metric_name = %metric_name,
                "[COLLECT] No data points found"
            );
            return Ok(None);
        }

        tracing::debug!(
            device_id = %device_id,
            metric_name = %metric_name,
            points_count = result.points.len(),
            "[COLLECT] Data points found"
        );

        let latest = &result.points[result.points.len() - 1];

        // Check if this is an image metric
        let is_image = is_image_metric(metric_name, &latest.value);
        let (image_url, image_base64, image_mime) = if is_image {
            extract_image_data(&latest.value)
        } else {
            (None, None, None)
        };

        // Build values JSON - construct once with all conditional fields
        let mut values_json = serde_json::json!({
            "value": latest.value,
            "timestamp": latest.timestamp,
            "points_count": result.points.len(),
            "time_range_minutes": time_range_minutes,
            "_is_image": is_image,
        });

        // Add image metadata if applicable
        if let Some(url) = &image_url {
            values_json["image_url"] = serde_json::json!(url);
        }
        if let Some(base64) = &image_base64 {
            values_json["image_base64"] = serde_json::json!(base64);
        }
        if let Some(mime) = &image_mime {
            values_json["image_mime_type"] = serde_json::json!(mime);
        }

        // Include history if configured and not an image
        if include_history && !is_image && result.points.len() > 1 {
            let history_limit = max_points.min(result.points.len());
            let start_idx = if result.points.len() > history_limit {
                result.points.len() - history_limit
            } else {
                0
            };

            let history_points = &result.points[start_idx..];

            // Use adaptive compression for compact output
            let compressed =
                heramind_storage::compress_series_adaptive(history_points, device_id, metric_name);
            values_json["history"] = compressed["series"].clone();
            values_json["history_count"] = serde_json::json!(history_points.len());

            // Include stats from compression output
            if let Some(stats) = compressed.get("stats").cloned() {
                values_json["stats"] = stats;
            }
        }

        Ok(Some(DataCollected {
            source: resource_id,
            data_type: metric_name.to_string(),
            values: values_json,
            timestamp,
        }))
    }

    pub(crate) async fn collect_device_data_parallel(
        &self,
        _agent: &AiAgent, // Reserved for future use
        device_ids: Vec<String>,
        bound_metrics: std::collections::HashMap<String, Vec<String>>,
        timestamp: i64,
    ) -> AgentResult<Vec<DataCollected>> {
        // If no device IDs, return empty data without requiring services
        if device_ids.is_empty() {
            tracing::debug!("No device resources to collect, returning empty data");
            return Ok(vec![]);
        }

        let device_service = self
            .device_service
            .as_ref()
            .ok_or(HeraMindError::validation(
                "Device service not available".to_string(),
            ))?;

        let storage = self
            .time_series_storage
            .clone()
            .ok_or(HeraMindError::validation(
                "Time series storage not available".to_string(),
            ))?;

        // Collect device info and metrics in parallel with timeout
        const QUERY_TIMEOUT_SECS: u64 = 10;

        let timeout_futures: Vec<_> = device_ids.into_iter()
            .map(|device_id| {
                let device_service = device_service.clone();
                let storage = storage.clone();
                let bound_metrics_for_device = bound_metrics.get(&device_id).cloned();
                async move {
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(QUERY_TIMEOUT_SECS),
                        Self::collect_single_device_data(
                            device_service,
                            storage,
                            &device_id,
                            bound_metrics_for_device,
                            timestamp
                        )
                    ).await {
                        Ok(result) => result,
                        Err(_) => {
                            tracing::warn!(device_id = %device_id, "Device data collection timed out after {}s", QUERY_TIMEOUT_SECS);
                            Ok(Vec::new()) // Return empty result on timeout
                        }
                    }
                }
            })
            .collect();

        let results = join_all(timeout_futures).await;
        let collected: Vec<_> = results
            .into_iter()
            .filter_map(|r| {
                if let Err(ref e) = r {
                    tracing::warn!(error = %e, "Data collection task failed");
                }
                r.ok()
            })
            .flat_map(|v| v.into_iter())
            .collect();
        Ok(collected)
    }

    /// Collect data from a single device resource.
    ///
    /// This collects:
    /// 1. Device metadata (device_info)
    pub(crate) async fn collect_single_device_data(
        device_service: Arc<DeviceService>,
        storage: Arc<heramind_storage::TimeSeriesStore>,
        device_id: &str,
        bound_metrics: Option<Vec<String>>,
        timestamp: i64,
    ) -> AgentResult<Vec<DataCollected>> {
        let mut data = Vec::new();

        // Get device info
        if let Some(device) = device_service.get_device(device_id) {
            let device_values = serde_json::json!({
                "device_id": device.device_id,
                "device_type": device.device_type,
                "name": device.name,
                "adapter_type": device.adapter_type,
            });

            data.push(DataCollected {
                source: device_id.to_string(),
                data_type: "device_info".to_string(),
                values: device_values,
                timestamp,
            });

            // Determine which metrics to collect
            // If bound_metrics is specified, only collect those; otherwise collect all
            let metrics: Vec<String> = if let Some(ref bound) = bound_metrics {
                tracing::debug!(
                    device_id = %device_id,
                    bound_metrics = ?bound,
                    "[COLLECT] Using bound metrics for device"
                );
                bound.clone()
            } else {
                // Get all available metrics for this device
                let all_metrics = storage
                    .list_metrics(&format!("device:{}", device_id))
                    .await
                    .unwrap_or_default();
                tracing::debug!(
                    device_id = %device_id,
                    metrics_count = all_metrics.len(),
                    "[COLLECT] Found all metrics for device (no binding)"
                );
                all_metrics
            };

            let end_time = chrono::Utc::now().timestamp();
            let start_time = end_time - (3600); // Last 1 hour for regular metrics

            // Image metrics to check separately (only collect one image)
            let image_metric_names = vec![
                "values.image",
                "image",
                "snapshot",
                "values.snapshot",
                "camera.image",
                "camera.snapshot",
                "picture",
                "values.picture",
                "frame",
                "values.frame",
            ];

            let mut image_found = false;
            let mut collected_image_parent: Option<String> = None;

            // Collect data for each metric
            for metric_name in metrics {
                // Skip if we already found an image and this is another image metric
                if image_found {
                    // Skip known image metric names
                    if image_metric_names.contains(&metric_name.as_str()) {
                        continue;
                    }
                    // Skip child paths of the already-collected image metric (e.g. "values.image.image_base64" under "values.image")
                    if let Some(ref parent) = collected_image_parent {
                        if metric_name.starts_with(&format!("{}.", parent)) {
                            continue;
                        }
                    }
                }

                // Query for data points
                let time_range = if image_metric_names.contains(&metric_name.as_str()) {
                    (end_time - 300, end_time) // 5 minutes for images
                } else {
                    (start_time, end_time) // 1 hour for regular metrics
                };

                if let Ok(result) = storage
                    .query_range(device_id, &metric_name, time_range.0, time_range.1, None)
                    .await
                {
                    if !result.points.is_empty() {
                        let latest = &result.points[result.points.len() - 1];
                        let is_image = is_image_metric(&metric_name, &latest.value);

                        if is_image {
                            let (image_url, image_base64, image_mime) =
                                extract_image_data(&latest.value);

                            let values_json = serde_json::json!({
                                "value": latest.value,
                                "timestamp": latest.timestamp,
                                "points_count": result.points.len(),
                                "_is_image": true,
                                "image_url": image_url,
                                "image_base64": image_base64,
                                "image_mime_type": image_mime,
                            });

                            data.push(DataCollected {
                                source: format!("{}:{}", device_id, metric_name),
                                data_type: metric_name.clone(),
                                values: values_json,
                                timestamp,
                            });

                            image_found = true;
                            collected_image_parent = Some(metric_name.clone());
                        } else {
                            // Regular metric - add latest value
                            let values_json = serde_json::json!({
                                "value": latest.value,
                                "timestamp": latest.timestamp,
                                "points_count": result.points.len(),
                            });

                            data.push(DataCollected {
                                source: format!("{}:{}", device_id, metric_name),
                                data_type: metric_name.clone(),
                                values: values_json,
                                timestamp,
                            });
                        }

                        tracing::debug!(
                            device_id = %device_id,
                            metric_name = %metric_name,
                            value = %latest.value,
                            "[COLLECT] Collected metric data"
                        );
                    }
                }
            }
        }

        tracing::debug!(
            device_id = %device_id,
            data_count = data.len(),
            "[COLLECT] Total data items collected for device"
        );

        Ok(data)
    }

    pub(crate) async fn collect_extension_metric_data_parallel(
        &self,
        _agent: &AiAgent,
        resources: Vec<AgentResource>,
        timestamp: i64,
    ) -> AgentResult<Vec<DataCollected>> {
        if resources.is_empty() {
            return Ok(Vec::new());
        }

        let registry = self
            .extension_registry
            .clone()
            .ok_or(HeraMindError::validation(
                "Extension registry not available".to_string(),
            ))?;

        let storage = self
            .time_series_storage
            .clone()
            .ok_or(HeraMindError::validation(
                "Time series storage not available".to_string(),
            ))?;

        // Collect extension metric data in parallel with timeout
        const QUERY_TIMEOUT_SECS: u64 = 10;

        let timeout_futures: Vec<_> = resources.into_iter()
            .map(|resource| {
                let resource_id = resource.resource_id.clone();
                let registry_clone = registry.clone();
                let storage_clone = storage.clone();

                async move {
                    // Normalize legacy format with duplicate "extension:" prefix
                    // Legacy: "extension:extension:ext_id:metric" -> Standard: "extension:ext_id:metric"
                    let normalized_resource_id = if resource_id.starts_with("extension:extension:") {
                        // Remove the duplicate "extension:" prefix
                        resource_id.replacen("extension:extension:", "extension:", 1)
                    } else {
                        resource_id.clone()
                    };

                    // Parse the resource_id using DataSourceId
                    // All extension metrics must use the DataSourceId format
                    let ds_id = match DataSourceId::parse(&normalized_resource_id) {
                        Some(id) if id.source_type == heramind_core::datasource::DataSourceType::Extension => id,
                        _ => {
                            tracing::warn!(
                                original_id = %resource_id,
                                normalized_id = %normalized_resource_id,
                                "Invalid extension metric resource ID format (must be extension:id:metric or extension:id:command.field)"
                            );
                            return Ok::<Option<DataCollected>, HeraMindError>(None);
                        }
                    };

                    // Extract parts for response
                    let extension_id = &ds_id.source_id;
                    let field_path = &ds_id.field_path;

                    tracing::debug!(
                        extension_id = %extension_id,
                        field_path = %field_path,
                        "[COLLECT] Querying extension metric"
                    );

                    // Query storage parts for historical data
                    let source_part = ds_id.source_part();
                    let metric_part = ds_id.metric_part();

                    // First, try to get current value from registry (most up-to-date)
                    let current_metric = tokio::time::timeout(
                        std::time::Duration::from_secs(QUERY_TIMEOUT_SECS),
                        registry_clone.get_current_metrics(extension_id)
                    ).await
                        .ok()
                        .and_then(|metric_values| {
                            metric_values.into_iter()
                                .find(|mv| mv.name == *field_path)
                        });

                    // Second, query historical data from storage
                    let end_time = chrono::Utc::now().timestamp();
                    let start_time = end_time - 3600; // Last 1 hour for historical data

                    let historical_result = tokio::time::timeout(
                        std::time::Duration::from_secs(QUERY_TIMEOUT_SECS),
                        storage_clone.query_range(&source_part, metric_part, start_time, end_time, None)
                    ).await;

                    let points_count = match &historical_result {
                        Ok(Ok(result)) => result.points.len(),
                        _ => 0,
                    };

                    // Build response combining current value and historical info
                    match (current_metric, historical_result) {
                        (Some(metric_value), Ok(Ok(_storage_result))) => {
                            // Has both current value and historical data
                            let json_value = metric_value_to_json(&metric_value.value);

                            // Detect image data so the analyzer can route via multimodal.
                            // Without this, an extension image metric (e.g.,
                            // extension:yolov8:values.image) would dump raw base64 into
                            // the text context and bloat the LLM prompt.
                            let is_image = is_image_metric(field_path, &json_value);
                            let (image_url, image_base64, image_mime) = if is_image {
                                extract_image_data(&json_value)
                            } else {
                                (None, None, None)
                            };

                            let mut values_json = serde_json::json!({
                                "extension_id": extension_id,
                                "value": json_value,
                                "timestamp": metric_value.timestamp,
                                "points_count": points_count,
                                "has_history": points_count > 1,
                                "_is_image": is_image,
                            });
                            if let Some(url) = &image_url {
                                values_json["image_url"] = serde_json::json!(url);
                            }
                            if let Some(b64) = &image_base64 {
                                values_json["image_base64"] = serde_json::json!(b64);
                                // Strip raw value — base64 bloats JSON serialization
                                // and the image is already represented as image_base64.
                                values_json.as_object_mut().map(|o| o.remove("value"));
                            }
                            if let Some(mime) = &image_mime {
                                values_json["image_mime_type"] = serde_json::json!(mime);
                            }

                            if is_image {
                                tracing::info!(
                                    extension_id = %extension_id,
                                    field_path = %field_path,
                                    has_base64 = image_base64.is_some(),
                                    has_url = image_url.is_some(),
                                    mime = ?image_mime,
                                    "[COLLECT] Extension metric identified as image"
                                );
                            }

                            tracing::debug!(
                                extension_id = %extension_id,
                                field_path = %field_path,
                                is_image,
                                points_count,
                                "[COLLECT] Extension metric found with historical data"
                            );

                            Ok(Some(DataCollected {
                                source: resource_id.clone(),
                                data_type: field_path.clone(),
                                values: values_json,
                                timestamp,
                            }))
                        }
                        (Some(metric_value), _) => {
                            // Only current value available, no historical data
                            let json_value = metric_value_to_json(&metric_value.value);

                            let is_image = is_image_metric(field_path, &json_value);
                            let (image_url, image_base64, image_mime) = if is_image {
                                extract_image_data(&json_value)
                            } else {
                                (None, None, None)
                            };

                            let mut values_json = serde_json::json!({
                                "extension_id": extension_id,
                                "value": json_value,
                                "timestamp": metric_value.timestamp,
                                "points_count": 1,
                                "has_history": false,
                                "_is_image": is_image,
                            });
                            if let Some(url) = &image_url {
                                values_json["image_url"] = serde_json::json!(url);
                            }
                            if let Some(b64) = &image_base64 {
                                values_json["image_base64"] = serde_json::json!(b64);
                                values_json.as_object_mut().map(|o| o.remove("value"));
                            }
                            if let Some(mime) = &image_mime {
                                values_json["image_mime_type"] = serde_json::json!(mime);
                            }

                            if is_image {
                                tracing::info!(
                                    extension_id = %extension_id,
                                    field_path = %field_path,
                                    has_base64 = image_base64.is_some(),
                                    has_url = image_url.is_some(),
                                    mime = ?image_mime,
                                    "[COLLECT] Extension metric identified as image (current only)"
                                );
                            }

                            tracing::debug!(
                                extension_id = %extension_id,
                                field_path = %field_path,
                                is_image,
                                "[COLLECT] Extension metric found (current only)"
                            );

                            Ok(Some(DataCollected {
                                source: resource_id.clone(),
                                data_type: field_path.clone(),
                                values: values_json,
                                timestamp,
                            }))
                        }
                        (None, Ok(Ok(storage_result))) if !storage_result.points.is_empty() => {
                            // No current value but historical data exists
                            let latest = &storage_result.points[storage_result.points.len() - 1];

                            let is_image = is_image_metric(field_path, &latest.value);
                            let (image_url, image_base64, image_mime) = if is_image {
                                extract_image_data(&latest.value)
                            } else {
                                (None, None, None)
                            };

                            let mut values_json = serde_json::json!({
                                "extension_id": extension_id,
                                "value": latest.value,
                                "timestamp": latest.timestamp,
                                "points_count": points_count,
                                "has_history": points_count > 1,
                                "_is_image": is_image,
                            });
                            if let Some(url) = &image_url {
                                values_json["image_url"] = serde_json::json!(url);
                            }
                            if let Some(b64) = &image_base64 {
                                values_json["image_base64"] = serde_json::json!(b64);
                                values_json.as_object_mut().map(|o| o.remove("value"));
                            }
                            if let Some(mime) = &image_mime {
                                values_json["image_mime_type"] = serde_json::json!(mime);
                            }

                            if is_image {
                                tracing::info!(
                                    extension_id = %extension_id,
                                    field_path = %field_path,
                                    has_base64 = image_base64.is_some(),
                                    has_url = image_url.is_some(),
                                    mime = ?image_mime,
                                    "[COLLECT] Extension metric identified as image (historical only)"
                                );
                            }

                            tracing::debug!(
                                extension_id = %extension_id,
                                field_path = %field_path,
                                is_image,
                                points_count,
                                "[COLLECT] Extension metric found in historical data only"
                            );

                            Ok(Some(DataCollected {
                                source: resource_id.clone(),
                                data_type: field_path.clone(),
                                values: values_json,
                                timestamp,
                            }))
                        }
                        _ => {
                            tracing::debug!(
                                extension_id = %extension_id,
                                field_path = %field_path,
                                "[COLLECT] Extension metric not found"
                            );
                            Ok(None)
                        }
                    }
                }
            })
            .collect();

        let results = join_all(timeout_futures).await;
        let collected: Vec<_> = results
            .into_iter()
            .filter_map(|r| {
                if let Err(ref e) = r {
                    tracing::warn!(error = %e, "Extension metric collection failed");
                }
                r.ok()
            })
            .flatten()
            .collect();

        tracing::debug!(
            collected_count = collected.len(),
            "[COLLECT] Extension metrics collected"
        );

        Ok(collected)
    }

    pub(crate) fn collect_memory_summary(
        &self,
        agent: &AiAgent,
        timestamp: i64,
    ) -> AgentResult<Vec<DataCollected>> {
        let mut results = Vec::new();

        // --- Memory summary from execution journal ---
        if !agent.memory.journal.records.is_empty() {
            let mut memory_summary = serde_json::Map::new();

            // Last outcome from most recent journal entry
            if let Some(last) = agent.memory.journal.records.last() {
                memory_summary.insert(
                    "last_conclusion".to_string(),
                    serde_json::json!(last.outcome),
                );
            }

            // Recent outcomes (up to 2)
            let conclusions: Vec<&str> = agent
                .memory
                .journal
                .records
                .iter()
                .rev()
                .take(2)
                .map(|r| r.outcome.as_str())
                .collect();
            if !conclusions.is_empty() {
                memory_summary.insert(
                    "recent_conclusions".to_string(),
                    serde_json::json!(conclusions),
                );
            }

            // Execution count = number of journal records
            memory_summary.insert(
                "total_executions".to_string(),
                serde_json::json!(agent.memory.journal.records.len()),
            );

            results.push(DataCollected {
                source: "memory".to_string(),
                data_type: "summary".to_string(),
                values: serde_json::to_value(memory_summary).unwrap_or_default(),
                timestamp,
            });
        }

        // Image analysis history is now handled by `build_history_context` in context.rs.
        // No need to inject it as DataCollected here.

        Ok(results)
    }

    /// Collect data including the triggering event data.
    pub(crate) async fn collect_data_with_event(
        &self,
        agent: &AiAgent,
        event_data: &EventTriggerData,
    ) -> AgentResult<Vec<DataCollected>> {
        let mut data = Vec::new();
        let _timestamp = chrono::Utc::now().timestamp(); // Reserved for future use

        // First, add the triggering event data directly
        let event_value_json = serde_json::to_value(&event_data.value).unwrap_or_default();

        // === DIAGNOSTIC LOG: raw event value ===
        {
            let value_kind = match &event_value_json {
                serde_json::Value::String(s) => {
                    let preview: String = s.chars().take(120).collect();
                    let has_data_prefix = s.starts_with("data:image/");
                    let has_http = s.starts_with("http://") || s.starts_with("https://");
                    let has_jpeg_magic = s.contains("/9j/");
                    let has_png_magic = s.contains("iVBORw0KGgo");
                    format!(
                        "String{{ len={}, has_data_prefix={}, has_http={}, has_jpeg_magic={}, has_png_magic={}, preview=\"{}\" }}",
                        s.len(), has_data_prefix, has_http, has_jpeg_magic, has_png_magic, preview
                    )
                }
                serde_json::Value::Object(obj) => {
                    let keys: Vec<&String> = obj.keys().collect();
                    format!("Object{{ keys={:?}, field_count={} }}", keys, obj.len())
                }
                other => format!("{:?}", other),
            };
            tracing::info!(
                target: "heramind::agent::event_value",
                source_type = %event_data.source.source_type,
                source_id = %event_data.source.source_id,
                field = %event_data.source.field,
                value_kind = %value_kind,
                "[DIAG] Raw event-triggered value"
            );
        }

        // Check if the event value is an image
        let is_image = is_image_metric(&event_data.source.field, &event_value_json);
        let (image_url, image_base64, image_mime) = if is_image {
            extract_image_data(&event_value_json)
        } else {
            (None, None, None)
        };

        // === DIAGNOSTIC LOG: image extraction result ===
        tracing::info!(
            target: "heramind::agent::event_value",
            source_id = %event_data.source.source_id,
            field = %event_data.source.field,
            is_image,
            has_url = image_url.is_some(),
            has_base64 = image_base64.is_some(),
            base64_len = image_base64.as_ref().map(|s| s.len()).unwrap_or(0),
            mime = ?image_mime,
            "[DIAG] Image extraction result"
        );

        // If the field is recognized as an image metric but extraction failed
        // (no URL and no base64), the event data is unusable — skip execution
        // rather than producing a meaningless empty analysis.
        if is_image && image_url.is_none() && image_base64.is_none() {
            tracing::warn!(
                source_id = %event_data.source.source_id,
                field = %event_data.source.field,
                "Event data is an image metric but extraction failed — skipping execution"
            );
            return Ok(vec![]);
        }

        let mut event_values = serde_json::json!({
            "value": event_data.value,
            "timestamp": event_data.timestamp,
            "_is_event_data": true,
        });

        // Add image metadata if applicable
        if is_image {
            event_values["_is_image"] = serde_json::json!(true);
            if let Some(ref url) = image_url {
                event_values["image_url"] = serde_json::json!(url);
            }
            if let Some(ref base64) = image_base64 {
                event_values["image_base64"] = serde_json::json!(base64);
            }
            if let Some(ref mime) = image_mime {
                event_values["image_mime_type"] = serde_json::json!(mime);
            }

            // Remove the raw `value` field — it may contain raw bytes with
            // control characters that break JSON serialization when the
            // execution record is later returned via the API.  The image is
            // already available as `image_base64` or `image_url`.
            if image_base64.is_some() || image_url.is_some() {
                event_values.as_object_mut().map(|o| o.remove("value"));
            }

            tracing::info!(
                source_id = %event_data.source.source_id,
                field = %event_data.source.field,
                has_url = image_url.is_some(),
                has_base64 = image_base64.is_some(),
                mime = ?image_mime,
                "Adding event-triggered image data to collection"
            );
        }

        data.push(DataCollected {
            source: format!(
                "{}:{}",
                event_data.source.source_id, event_data.source.field
            ),
            data_type: event_data.source.field.clone(),
            values: event_values,
            timestamp: event_data.timestamp,
        });

        // Add device metadata so the LLM knows which device triggered this event
        if event_data.source.source_type == "device" {
            if let Some(ref device_service) = self.device_service {
                if let Some(device) = device_service.get_device(&event_data.source.source_id) {
                    let device_meta = serde_json::json!({
                        "device_id": device.device_id,
                        "device_type": device.device_type,
                        "name": device.name,
                        "adapter_type": device.adapter_type,
                        "_is_event_device": true,
                    });

                    data.push(DataCollected {
                        source: event_data.source.source_id.clone(),
                        data_type: "device_info".to_string(),
                        values: device_meta,
                        timestamp: event_data.timestamp,
                    });
                }
            }
        }

        // Then collect other data from regular sources
        let regular_data = self.collect_data(agent).await?;

        // Build the set of source keys already covered by the event data.
        // The event value's image may be expanded into sub-fields (e.g.
        // "values.image" → "values.image.image_base64"), so we match by
        // prefix to catch all variants.
        let event_source_device = &event_data.source.source_id;
        let event_source_field = &event_data.source.field;

        // Add regular data (excluding duplicates)
        for item in regular_data {
            // Skip if it's the placeholder guidance from collect_data
            if item.data_type == "info" && item.source == "system" {
                continue;
            }
            // Skip exact source match (e.g. "device:values.image")
            if item.source == format!("{}:{}", event_source_device, event_source_field) {
                continue;
            }
            // Skip if this regular item is from the same device AND its
            // data_type is the event field or a sub-field of it (e.g.
            // "values.image" matches event field "values.image", and
            // "values.image.image_base64" is a sub-field → skip).
            if item
                .source
                .starts_with(&format!("{}:", event_source_device))
            {
                let item_metric = &item.source[event_source_device.len() + 1..];
                if item_metric == event_source_field
                    || item_metric.starts_with(&format!("{}.", event_source_field))
                {
                    continue;
                }
            }
            data.push(item);
        }

        tracing::debug!(
            agent_id = %agent.id,
            event_source = %event_data.source.source_id,
            event_field = %event_data.source.field,
            total_data_count = data.len(),
            event_is_image = is_image,
            "Collected data including event trigger"
        );

        Ok(data)
    }
}

/// Extract image data from a metric value.
/// Returns (image_url, base64_data, mime_type).
pub(crate) fn extract_image_data(
    value: &serde_json::Value,
) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(s) = value.as_str() {
        if s.starts_with("http://") || s.starts_with("https://") {
            tracing::info!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: matched URL branch");
            (Some(s.to_string()), None, None)
        } else if s.starts_with("data:image/") {
            tracing::info!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: matched data:image/ branch, len={}", s.len());
            if let Some(rest) = s.strip_prefix("data:image/") {
                let parts: Vec<&str> = rest.splitn(2, ';').collect();
                if parts.len() == 2 {
                    // Canonicalize known subtypes (incl. jpg→jpeg alias).
                    // For unknown subtypes like heic/avif, preserve the
                    // literal "image/<subtype>" so downstream can decide.
                    let subtype = parts[0].to_ascii_lowercase();
                    let mime_type = crate::image_utils::normalize_mime_subtype(&subtype)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("image/{}", subtype));
                    if let Some(data) = parts[1].strip_prefix("base64,") {
                        (None, Some(data.to_string()), Some(mime_type))
                    } else {
                        (None, Some(parts[1].to_string()), Some(mime_type))
                    }
                } else {
                    (None, Some(rest.to_string()), Some("image/jpeg".to_string()))
                }
            } else {
                (None, Some(s.to_string()), Some("image/jpeg".to_string()))
            }
        } else if s.len() > 100 {
            // Detect image MIME from base64 magic prefix. Use starts_with
            // (NOT contains) so a text string that merely mentions "/9j/"
            // somewhere in the middle isn't misclassified as an image.
            let mime_type = crate::image_utils::infer_mime_from_base64_prefix(s);
            if let Some(mt) = mime_type {
                tracing::info!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: matched magic-bytes branch, len={}, mime={}", s.len(), mt);
                (None, Some(s.to_string()), Some(mt.to_string()))
            } else {
                tracing::warn!(
                    target: "heramind::agent::event_value",
                    len = s.len(),
                    starts_with = ?s.chars().take(40).collect::<String>(),
                    "[DIAG] extract_image_data: STRING branch FALLTHROUGH (no match) — value not recognized as image"
                );
                (None, None, None)
            }
        } else {
            tracing::warn!(
                target: "heramind::agent::event_value",
                len = s.len(),
                starts_with = ?s.chars().take(40).collect::<String>(),
                has_jpeg_magic = s.contains("/9j/"),
                has_png_magic = s.contains("iVBORw0KGgo"),
                "[DIAG] extract_image_data: STRING branch FALLTHROUGH (no match) — value not recognized as image"
            );
            (None, None, None)
        }
    } else if let Some(obj) = value.as_object() {
        let keys: Vec<&String> = obj.keys().collect();
        tracing::info!(target: "heramind::agent::event_value", keys = ?keys, "[DIAG] extract_image_data: examining Object");
        if let Some(url) = obj
            .get("image_url")
            .or(obj.get("url"))
            .and_then(|v| v.as_str())
        {
            tracing::info!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: matched Object URL field");
            return (Some(url.to_string()), None, None);
        }
        if let Some(base64) = obj
            .get("base64")
            .or(obj.get("data"))
            .or(obj.get("image_data"))
            .or(obj.get("image_base64"))
            .and_then(|v| v.as_str())
        {
            tracing::info!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: matched Object base64 field, len={}", base64.len());
            // Prefer the explicit mime_type/type/image_mime_type field; otherwise infer from
            // base64 magic prefix; only fall back to jpeg if neither works.
            let mime = obj
                .get("image_mime_type")
                .or(obj.get("mime_type"))
                .or(obj.get("type"))
                .and_then(|v| v.as_str())
                .map(|m| m.to_string())
                .or_else(|| {
                    crate::image_utils::infer_mime_from_base64_prefix(base64).map(|s| s.to_string())
                })
                .unwrap_or_else(|| "image/jpeg".to_string());
            return (None, Some(base64.to_string()), Some(mime));
        }
        tracing::warn!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: OBJECT branch FALLTHROUGH — no recognized image keys");
        (None, None, None)
    } else {
        tracing::warn!(target: "heramind::agent::event_value", "[DIAG] extract_image_data: value is neither String nor Object");
        (None, None, None)
    }
}

/// Check if a metric value contains image data.
pub(crate) fn is_image_metric(metric_name: &str, value: &serde_json::Value) -> bool {
    // Check metric name for image-related keywords
    let name_indicates_image = metric_name.to_lowercase().contains("image")
        || metric_name.to_lowercase().contains("snapshot")
        || metric_name.to_lowercase().contains("photo")
        || metric_name.to_lowercase().contains("picture")
        || metric_name.to_lowercase().contains("camera")
        || metric_name.to_lowercase().contains("video")
        || metric_name.to_lowercase().contains("frame");

    if name_indicates_image {
        return true;
    }

    // Check value for URL or base64 data
    if let Some(s) = value.as_str() {
        // Check for URL
        if s.starts_with("http://") || s.starts_with("https://") {
            return true;
        }
        // Check for base64 image data
        if s.starts_with("data:image/") {
            return true;
        }
        // Check for common base64 prefixes without data URL scheme.
        // IMPORTANT: use starts_with (not contains) so a text string that
        // merely mentions "/9j/" somewhere in the middle isn't misclassified.
        if s.len() > 100 && crate::image_utils::infer_mime_from_base64_prefix(s).is_some() {
            return true;
        }
        false
    } else if let Some(obj) = value.as_object() {
        // Check for image_url, url, base64, or data fields
        obj.contains_key("image_url")
            || obj.contains_key("url")
            || obj.contains_key("base64")
            || obj.contains_key("data")
            || obj.contains_key("image_data")
    } else {
        false
    }
}
