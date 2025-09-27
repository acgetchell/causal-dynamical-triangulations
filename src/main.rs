use causal_dynamical_triangulations::Config;

fn main() {
    let config = Config::build();
    let _triangulation = causal_dynamical_triangulations::run(&config);
}
