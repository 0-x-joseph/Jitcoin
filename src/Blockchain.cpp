#include "Blockchain.h"
#include "Block.h"

Blockchain::Blockchain() {
  _vchain.emplace_back(Block(0, "Genesis Block"));
  _ndifficulty = 6;
}

Blockchain::~Blockchain() {}

Block Blockchain::_get_last_block() const { return _vchain.back(); }

void Blockchain::add_block(Block b_new) {
  b_new.sprev_hash = _get_last_block().get_hash();
  b_new.mine_block(_ndifficulty);
  _vchain.push_back(b_new);
}
