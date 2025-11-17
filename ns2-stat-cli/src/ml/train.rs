use burn::data::dataloader::DataLoaderBuilder;
use burn::optim::AdamConfig;
use burn::prelude::*;
use burn::record::{CompactRecorder, Recorder};
use burn::tensor::backend::AutodiffBackend;
use burn::train::metric::LossMetric;
use burn::train::{LearnerBuilder, LearningStrategy, RegressionOutput, TrainOutput, TrainStep, ValidStep};

use crate::ml::data::{GameBatch, GameBatcher, GameDataset, GameInput, GameOutput, decode_output, encode_input};
use crate::ml::model::{Model, ModelConfig};

impl<B: AutodiffBackend> TrainStep<GameBatch<B>, RegressionOutput<B>> for Model<B> {
    fn step(&self, batch: GameBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let item = self.forward_regression(batch.player_ids, batch.input, batch.targets);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<GameBatch<B>, RegressionOutput<B>> for Model<B> {
    fn step(&self, batch: GameBatch<B>) -> RegressionOutput<B> {
        self.forward_regression(batch.player_ids, batch.input, batch.targets)
    }
}

#[derive(Config, Debug)]
pub struct TrainingConfig {
    model: ModelConfig,
    optimizer: AdamConfig,
    #[config(default = 10)]
    pub num_epochs: usize,
    #[config(default = 10)]
    pub batch_size: usize,
    #[config(default = 4)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    #[config(default = 1.0e-4)]
    pub learning_rate: f64,
}

fn create_artifact_dir(artifact_dir: &str) {
    // Remove existing artifacts before to get an accurate learner summary
    std::fs::remove_dir_all(&artifact_dir).ok();
    std::fs::create_dir_all(&artifact_dir).ok();
}

pub fn train<B: AutodiffBackend>(
    artifact_dir: &str,
    config: TrainingConfig,
    device: &B::Device,
    train_dataset: GameDataset,
    test_dataset: GameDataset,
) {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");

    let dataloader_train = DataLoaderBuilder::new(GameBatcher)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(train_dataset);

    let dataloader_test = DataLoaderBuilder::new(GameBatcher)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(test_dataset);

    let learner = LearnerBuilder::new(artifact_dir)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .learning_strategy(LearningStrategy::SingleDevice(device.clone()))
        .num_epochs(config.num_epochs)
        .summary()
        .build(
            config.model.init::<B>(device),
            config.optimizer.init(),
            config.learning_rate,
        );

    let result = learner.fit(dataloader_train, dataloader_test);

    result
        .model
        .save_file(format!("{artifact_dir}/model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}

pub fn infer<B: Backend<FloatElem = f32>>(artifact_dir: &str, device: &B::Device, game_input: GameInput) -> GameOutput {
    let config = TrainingConfig::load(format!("{artifact_dir}/config.json"))
        .expect("Config should exist for the model; run train first");
    let record = CompactRecorder::new()
        .load(format!("{artifact_dir}/model").into(), device)
        .expect("Trained model should exist; run train first");

    let model = config.model.init::<B>(device).load_record(record);

    let (player_ids, input) = encode_input(device, game_input);
    let output = model.forward(player_ids.unsqueeze_dim(0), input.unsqueeze_dim(0));
    decode_output(output.squeeze_dim(0))
}
