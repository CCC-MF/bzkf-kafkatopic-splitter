/*
 * MIT License
 *
 * Copyright (c) 2023 Comprehensive Cancer Center Mainfranken
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::env;
use std::error::Error;
use std::fmt::{Debug as FmtDebug, Display, Formatter};
use std::time::Duration;

use log::LevelFilter::{Debug, Info};
use log::{debug, error, info, warn};
use rdkafka::consumer::{Consumer, ConsumerContext, Rebalance, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientConfig, ClientContext, Message, TopicPartitionList};
use serde::Deserialize;
use simple_logger::SimpleLogger;

use crate::AppError::ConnectionError;

struct CustomContext;

impl ClientContext for CustomContext {}

type LoggingConsumer = StreamConsumer<CustomContext>;

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        debug!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        debug!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        debug!("Committing offsets: {:?}", result);
    }
}

enum AppError {
    ConnectionError(String),
}

impl Error for AppError {}

impl FmtDebug for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ConnectionError(s) => write!(f, "ConnectionError: {}", s),
        }
    }
}

#[derive(Deserialize)]
struct PayloadWithYear {
    #[serde(rename = "YEAR")]
    year: u16,
}

#[derive(Deserialize)]
struct MessageWithPayload {
    #[serde(rename = "payload")]
    inner_payload: PayloadWithYear,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    SimpleLogger::new().with_level(Debug).init().unwrap();

    #[cfg(not(debug_assertions))]
    SimpleLogger::new().with_level(Info).init().unwrap();

    let context = CustomContext;

    let group_id = env::var("KAFKA_GROUP_ID").unwrap_or("bzkf-kafkatopic-splitter".into());
    let boostrap_servers = env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or("kafka:9094".into());
    let source_topic = env::var("KAFKA_SOURCE_TOPIC").unwrap_or("onkostar.MELDUNG_EXPORT".into());
    let topic_prefix =
        env::var("KAFKA_DESTINATION_TOPIC_PREFIX").unwrap_or("onkostar.MELDUNG_EXPORT.".into());

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", boostrap_servers.as_str())
        .set("auto.offset.reset", "earliest")
        .create_with_context(context)
        .expect("Kafka consumer created");

    consumer
        .subscribe([source_topic.as_str()].as_ref())
        .map_err(|e| ConnectionError(e.to_string()))?;

    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", boostrap_servers.as_str())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    info!("Application started");

    loop {
        match consumer.recv().await {
            Ok(msg) => match msg.payload_view::<str>() {
                Some(Ok(s)) => match serde_json::from_str::<MessageWithPayload>(s) {
                    Ok(mwy) => {
                        debug!(
                            "Message for topic '{}{}'",
                            topic_prefix, mwy.inner_payload.year
                        );
                        let _ = producer
                            .send(
                                FutureRecord::to(
                                    format!("{}{}", topic_prefix, mwy.inner_payload.year).as_str(),
                                )
                                .payload(s)
                                .key(msg.key().unwrap()),
                                Duration::from_secs(0),
                            )
                            .await;
                    }
                    Err(err) => error!("Cannot deserialize message: {}", err),
                },
                Some(Err(err)) => error!("Cannot get payload: {}", err),
                _ => warn!("Unable to consume payload"),
            },
            Err(err) => error!("Kafka Error: {}", err),
        }
    }
}
