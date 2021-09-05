use lambda::{
    core::application::create_lambda_application,
    core::application::start_application,
};
fn main() {
    let lambda = create_lambda_application();
    println!("{} was just created", lambda.get_name());
    start_application(lambda);
}