use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::Dataset;
use burn::prelude::*;

pub const MAX_PLAYERS_PER_TEAM: usize = 10;
pub const NUM_PLAYER_FEATURES: usize = 2;
pub const FEATURES_PER_TEAM: usize = MAX_PLAYERS_PER_TEAM * NUM_PLAYER_FEATURES;

const SECONDS_PER_MINUTE: f32 = 60.0;

#[derive(Clone, Debug)]
pub struct GameData {
    pub input: GameInput,
    pub output: GameOutput,
}

#[derive(Clone, Debug)]
pub struct GameInput {
    pub aliens: Vec<PlayerData>,
    pub marines: Vec<PlayerData>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerData {
    pub id: usize,
    /// average score per minute
    pub avg_score: f32,
    pub kd: f32,
}

impl PlayerData {
    pub fn new(id: usize, avg_score: f32, kd: f32) -> Self {
        fn make_finite(x: f32) -> f32 {
            if x.is_finite() {
                x
            } else {
                0.0
            }
        }

        Self {
            id,
            avg_score: make_finite(avg_score * SECONDS_PER_MINUTE),
            kd: make_finite(kd),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameOutput {
    //pub winner: WinningTeam,
    pub round_length: f32,
}

#[derive(Clone)]
pub struct GameDataset {
    games: Vec<GameData>,
}

impl GameDataset {
    pub fn new(games: Vec<GameData>) -> Self {
        Self { games }
    }
}

impl Dataset<GameData> for GameDataset {
    fn get(&self, index: usize) -> Option<GameData> {
        self.games.get(index).map(GameData::clone)
    }

    fn len(&self) -> usize {
        self.games.len()
    }
}

pub struct GameBatcher;

#[derive(Clone, Debug)]
pub struct GameBatch<B: Backend> {
    pub player_ids: Tensor<B, 4, Int>,
    pub input: Tensor<B, 4>,
    pub targets: Tensor<B, 2>,
}

impl<B: Backend> Batcher<B, GameData, GameBatch<B>> for GameBatcher {
    fn batch(&self, items: Vec<GameData>, device: &B::Device) -> GameBatch<B> {
        let (player_ids, input, targets) = items
            .into_iter()
            .map(|game_data| {
                let (player_ids, input) = encode_input(device, game_data.input);
                let target = encode_output(device, game_data.output);
                (player_ids, input, target)
            })
            .collect();
        GameBatch {
            player_ids: Tensor::stack(player_ids, 0),
            input: Tensor::stack(input, 0),
            targets: Tensor::stack(targets, 0),
        }
    }
}

pub fn encode_input<B: Backend>(device: &B::Device, game_input: GameInput) -> (Tensor<B, 3, Int>, Tensor<B, 3>) {
    let mut player_ids = Vec::new();
    let mut input = Vec::new();
    for p in game_input.aliens {
        player_ids.push(p.id);
        input.extend([p.avg_score, p.kd]);
    }
    player_ids.resize(MAX_PLAYERS_PER_TEAM, 0);
    input.resize(FEATURES_PER_TEAM, 0.0);
    for p in game_input.marines {
        player_ids.push(p.id);
        input.extend([p.id as f32, p.avg_score, p.kd]);
    }
    player_ids.resize(2 * MAX_PLAYERS_PER_TEAM, 0);
    input.resize(2 * FEATURES_PER_TEAM, 0.0);
    (
        Tensor::<B, 1, Int>::from_ints(player_ids.as_slice(), device).reshape([2, MAX_PLAYERS_PER_TEAM, 1]),
        Tensor::<B, 1>::from_floats(input.as_slice(), device).reshape([2, MAX_PLAYERS_PER_TEAM, NUM_PLAYER_FEATURES]),
    )
}

pub fn encode_output<B: Backend>(device: &B::Device, game_output: GameOutput) -> Tensor<B, 1> {
    Tensor::from_floats([game_output.round_length], device)
}

pub fn decode_output<B: Backend<FloatElem = f32>>(output: Tensor<B, 1>) -> GameOutput {
    GameOutput { round_length: output.into_scalar() }
}
