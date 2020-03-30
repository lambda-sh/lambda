namespace engine {
  __attribute__((visibility ("default"))) void Print();
}

int main() {
  engine::Print();    
  return 0;
}
