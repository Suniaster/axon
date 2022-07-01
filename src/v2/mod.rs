use nalgebra::{SVector, SMatrix, DMatrix};
use rand::Rng;
use super::activations::{Activation, ActivationType};

pub struct Neuron<const D: usize>{
    pub weights: SVector<f64, D>,
    pub bias: f64,
}

impl<const D:usize> Neuron<D> {
    pub fn new(bias: f64) -> Neuron<D> {
        let zeros = vec![0.0; D];
        let weights = SVector::from_vec(zeros);
        Neuron {
            weights,
            bias: bias,
        }
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        self.weights = SVector::from_fn(|_,_| rng.gen::<f64>());
    }

    pub fn normalize(&mut self) {
        let ones = vec![1.; D];
        self.weights = SVector::from_vec(ones);
        self.bias = 0.0;
    }

    pub fn foward(&self, inputs: &SVector<f64, D>) -> f64 {
        inputs.dot(&self.weights) + self.bias
    }
}

trait LayerFormat {
    const F: usize;
}  

pub trait NetLayer{
    fn foward(&self, inputs: Vec<f64>) -> Vec<f64>;
    fn format(&self) -> (usize, usize);

    fn get_weights(&self) -> Vec<Vec<f64>>;
    fn get_errors(&self) -> Vec<f64>;
    fn get_last_result(&self) -> Vec<f64>;

    fn foward_mut(&mut self, inputs: Vec<f64>) -> Vec<f64>;
    fn backward_output(&mut self, expected: Vec<f64>) -> Vec<f64>;
    fn backward(&mut self, nl_ws: Vec<Vec<f64>>, nl_deltas: Vec<f64>) -> Vec<f64>;
    fn update_layer(&mut self, pl_result: Vec<f64>, l_rate: f64);
}

pub struct DenseLayer<const IN_FMT: usize, const OUT_FMT: usize> {
    neurons: Vec<Neuron<IN_FMT>>,
    weights_mat: SMatrix<f64, OUT_FMT, IN_FMT>,
    bias_vec: SVector<f64, OUT_FMT>,

    activation: Activation,
    last_result: SVector<f64, OUT_FMT>,
    error: SVector<f64, OUT_FMT>
}

impl<const IN_FMT:usize, const OUT_FMT:usize> DenseLayer<IN_FMT, OUT_FMT>{
    pub fn new() -> DenseLayer<IN_FMT, OUT_FMT> {
        let mut neurons = Vec::new();
        for _ in 0..OUT_FMT {
            neurons.push(Neuron::new(0.0));
        }
        DenseLayer {
            neurons,
            weights_mat: SMatrix::<f64, OUT_FMT, IN_FMT>::zeros(),
            bias_vec: SVector::<f64, OUT_FMT>::zeros(),
            activation: Activation::create(ActivationType::Default),

            last_result: SVector::<f64, OUT_FMT>::zeros(),
            error: SVector::zeros()
        }
    }

    pub fn set_activation(&mut self, activation: ActivationType) {
        self.activation = Activation::create(activation);
    }

    pub fn randomize(&mut self) {
        for n in &mut self.neurons {
            n.randomize();
        }
        self.update_weights_mat();
    }

    pub fn normalize(&mut self) {
        for n in &mut self.neurons {
            n.normalize();
        }
        self.update_weights_mat();
    }

    pub fn update_weights_mat(&mut self) {
        for (i, n) in self.neurons.iter().enumerate() {
            for (j, w) in n.weights.iter().enumerate() {
                self.weights_mat[(i, j)] = *w;
            }
        }
    }

}

impl<const I:usize, const O:usize> NetLayer for DenseLayer<I,O> {
    fn foward(&self, inputs: Vec<f64>) -> Vec<f64> {
        let input_vec:SVector<f64, I> = SVector::from_vec(inputs);
        let out: SVector<f64, O> = self.weights_mat * input_vec + self.bias_vec;
        let out:Vec<f64> = out.data.0[0].to_vec();
        out.iter().map(|o| (self.activation.f)(o)).collect()
    }

    fn foward_mut(&mut self, inputs: Vec<f64>) -> Vec<f64> {
        let res = self.foward(inputs);
        self.last_result = SVector::from_vec(res);
        self.last_result.data.0[0].to_vec()
    }

    fn format(&self) -> (usize, usize) {
        (I, O)
    }

    fn get_weights(&self) -> Vec<Vec<f64>> {
        let mut weights = Vec::new();
        for n in &self.neurons {
            weights.push(n.weights.data.0[0].to_vec());
        }
        weights
    }

    fn get_last_result(&self) -> Vec<f64> {
        self.last_result.data.0[0].to_vec()
    }

    fn backward_output(&mut self, expected: Vec<f64>) -> Vec<f64> {
        let expected_vec =  SVector::from_vec(expected);
        
        self.error = expected_vec - self.last_result;
        self.error = self.error.component_mul(&self.error); // (expected - output)^2

        let derivatives = self.last_result.map(|o| (self.activation.d)(&o));
        self.error = self.error.component_mul(&derivatives); // (expected - output)^2 * derivative(output)

        return self.error.data.0[0].to_vec();
    }

    fn backward(&mut self, nl_ws: Vec<Vec<f64>>, nl_deltas: Vec<f64>) -> Vec<f64> {
        let derivatives = self.last_result.map(|o| (self.activation.d)(&o)); 
        
        let w_format = (nl_ws.len(), nl_ws[0].len());
        let w_mat:DMatrix<f64> = DMatrix::from_vec(w_format.0, w_format.1, nl_ws.into_iter().flatten().collect());
        
        let nl_deltas:DMatrix<f64> = DMatrix::from_vec(nl_deltas.len(), 1, nl_deltas);

        let e = w_mat.transpose() * nl_deltas;
        self.error = e.component_mul(&derivatives);
        
        return self.error.data.0[0].to_vec();
    }

    fn update_layer(&mut self, pl_result: Vec<f64>, l_rate: f64) {
        for (i, n) in self.neurons.iter().enumerate() {
            for (j, _) in n.weights.iter().enumerate() {
                self.weights_mat[(i,j)] -= l_rate * self.error[i] * pl_result[j];
            }
            self.bias_vec[i] -= l_rate * self.error[i];
        }
    }

    fn get_errors(&self) -> Vec<f64> {
        self.error.data.0[0].to_vec()
    }
}


/********** Network *********/

pub struct ArtificialNetwork {
    layers: Vec<Box<dyn NetLayer>>
}

impl ArtificialNetwork {
    pub fn new() -> ArtificialNetwork {
        ArtificialNetwork {
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, layer: Box<dyn NetLayer>) -> &mut Self {
        self.verify_new_layer(&layer);
        self.layers.push(layer);
        self
    }

    
    pub fn foward(&self, inputs: Vec<f64>) -> Vec<f64> {
        let mut inputs = inputs;
        for layer in &self.layers {
            inputs = layer.foward(inputs);
        }
        inputs
    }

    pub fn train(&mut self, inputs: Vec<Vec<f64>>, expected: Vec<Vec<f64>>, l_rate: f64, epochs: usize) -> (f64, f64) {
        let loss1 = self.learn(inputs[0].clone(), expected[0].clone(), l_rate);
        let mut loss2 = 0.;
        let train_data_size = inputs.len();
        for i in 0..epochs {
            
            loss2 = self.learn(inputs[i % train_data_size].clone(), expected[i % train_data_size].clone(), l_rate);
            print!("\rEpoch: {} \t\t| loss: {:?}", i, loss2);
        }
        println!();
        return (loss1, loss2);
    }

    fn foward_mut(&mut self, inputs: Vec<f64>) -> Vec<f64> {
        let mut inputs = inputs;
        for layer in &mut self.layers {
            inputs = layer.foward_mut(inputs);
        }
        inputs
    }

    fn verify_new_layer(&self, new_layer: &Box<dyn NetLayer>){
        let layer_len = self.layers.len();
        if layer_len > 0 {
            let last_layer_format = self.layers[layer_len - 1].format();
            if last_layer_format.1 != new_layer.format().0 {
                panic!("Layer {:?} cannot project to layer {:?}", last_layer_format, new_layer.format().0);
            }
        }
    }

    fn backward(&mut self, expected: Vec<f64>) {
        let size = self.layers.len();

        let mut expected = expected;
        expected = self.layers[size - 1].backward_output(expected);

        let mut ll_ws = self.layers[size - 1].get_weights();

        for i in (0..self.layers.len()-1).rev() {
            expected = self.layers[i].backward(ll_ws, expected);
            ll_ws = self.layers[i].get_weights();
        }
    }

    fn learn(&mut self, inputs:Vec<f64>, expected: Vec<f64>, l_rate: f64) -> f64{
        let inp_clone = inputs.clone();
        let exp_clone = expected.clone();

        self.foward_mut(inputs);
        self.backward(expected);
        
        for i in &mut (1..self.layers.len()) {
            let previous_layer_result = self.layers[i-1].get_last_result();
            self.layers[i].update_layer(previous_layer_result, l_rate);
        }
    
        return self.get_loss(inp_clone, exp_clone);
    }

    fn get_loss(&self, input: Vec<f64>, expected: Vec<f64>) -> f64 {
        let output_format = self.layers[self.layers.len() - 1].format();
        let out = self.foward(input);

        let expct = DMatrix::from_vec(output_format.1, 1, expected);
        let out = DMatrix::from_vec(output_format.1, 1, out);
        
        let sub = out - expct;  
        let loss = sub.component_mul(&sub); // (expected - output)^2
        return loss.sum();
    }
}