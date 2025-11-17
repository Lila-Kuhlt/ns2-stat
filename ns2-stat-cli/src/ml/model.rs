use burn::nn::loss::{MseLoss, Reduction};
use burn::nn::{Embedding, EmbeddingConfig, Linear, LinearConfig, Relu};
use burn::prelude::*;
use burn::train::RegressionOutput;

use crate::ml::data::NUM_PLAYER_FEATURES;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    player_embedding: Embedding<B>,
    player_layer: Linear<B>,
    output: Linear<B>,
    activation: Relu,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    dim_player_embedding: usize,
    size_player_layer: usize,
}

impl ModelConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        Model {
            player_embedding: EmbeddingConfig::new(64, self.dim_player_embedding).init(device),
            player_layer: LinearConfig::new(self.dim_player_embedding + NUM_PLAYER_FEATURES, self.size_player_layer).init(device),
            output: LinearConfig::new(self.size_player_layer * 2, 1).init(device),
            activation: Relu::new(),
        }
    }
}

impl<B: Backend> Model<B> {
    pub fn forward(&self, player_ids: Tensor<B, 4, Int>, input: Tensor<B, 4>) -> Tensor<B, 2> {
        let [batch_size, teams, players, _] = player_ids.dims();
        let x = Tensor::cat(vec![
            self.player_embedding.forward(player_ids.flatten(1, 3)).reshape([batch_size as i64, teams as i64, players as i64, -1]),
            input,
        ], 3);
        let x = self.player_layer.forward(x);
        let x = self.activation.forward(x);
        // average players
        let x = x.mean_dim(2).flatten(1, 3);
        let x = self.output.forward(x);
        let x = self.activation.forward(x);
        x
    }

    pub fn forward_regression(&self, player_ids: Tensor<B, 4, Int>, input: Tensor<B, 4>, targets: Tensor<B, 2>) -> RegressionOutput<B> {
        let output = self.forward(player_ids, input);
        let loss = MseLoss::new().forward(output.clone(), targets.clone(), Reduction::Auto);
        RegressionOutput::new(loss, output, targets)
    }
}
