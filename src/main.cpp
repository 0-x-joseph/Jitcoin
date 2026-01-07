#include "Blockchain.h"
#include <iostream>

int main() {
  Blockchain jchain = Blockchain();

  std::cout << "Mining Block 0..." << std::endl;
  jchain.add_block(Block(1, "Block 1 Data"));

  std::cout << "Mining Block 1..." << std::endl;
  jchain.add_block(Block(2, "Block 2 Data"));

  std::cout << "Mining Block 2..." << std::endl;
  jchain.add_block(Block(3, "Block 3 Data"));

  return (0);
}
