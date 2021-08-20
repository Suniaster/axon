use super::layer::*;
use super::perceptron::*;
use ndarray::Array1;

pub struct NeuralNet {
    layers: Vec<Layer>,
}

impl NeuralNet {
    pub fn from_format(format: &[i32]) -> NeuralNet {
        let mut layers: Vec<Layer> = Vec::new();
        let mut input_shape: i32 = 1;
        for i in 0..format.len() {
            if i == 0 {
                input_shape = format[0];
                continue;
            }
            layers.push(Layer::new_dense(input_shape as usize, format[i] as usize));
            input_shape = format[i];
        }
        return NeuralNet { layers };
    }

    pub fn create(layers: Vec<Layer>) -> NeuralNet{
        return NeuralNet { layers };
    }

    pub fn foward(&self, input: &Array1<f64>) -> Array1<f64> {
        let mut iterator = self.layers.iter();
        let input_layer = iterator.next().expect("NN with no input layer");

        let mut layer_input = input_layer.foward(input);

        for hidden_layer in iterator {
            layer_input = hidden_layer.foward(&layer_input);
        }

        return layer_input;
    }
}
