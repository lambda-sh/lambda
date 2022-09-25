use super::Instance;

pub trait Resource<ResourceOwner> {
  fn delete(self, instance: &mut ResourceOwner);
}
