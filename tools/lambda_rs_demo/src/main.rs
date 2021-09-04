use lambda::{
    core::create_lambda_application,
    core::start_application,
};
fn main() {
    let lambda = create_lambda_application();
    println!("{} was just created", lambda.get_name());
    start_application(lambda);
}