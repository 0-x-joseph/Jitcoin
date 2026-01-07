/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Block.cpp                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ybouryal <ybouryal@student.1337.ma>        +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2026/01/07 02:24:14 by ybouryal          #+#    #+#             */
/*   Updated: 2026/01/07 03:17:58 by ybouryal         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "sha256.h"
#include <iostream>
#include <sstream>
#include <string>

#include "Block.h"

Block::Block(uint32_t nindexin, const std::string &sdatain)
    : _nindex(nindexin), _sdata(sdatain), _nnonce(-1) {}

Block::~Block() {}

std::string Block::get_hash() { return _shash; }

void Block::mine_block(uint32_t ndifficulty) {
  char cstr[ndifficulty + 1];

  for (int i = 0; i < ndifficulty; i++) {
    cstr[i] = '0';
  }
  cstr[ndifficulty] = '\0';

  std::string str(cstr);

  do {
    _nnonce++;
    _shash = _calc_hash();
  } while (_shash.substr(0, ndifficulty) != str);
  std::cout << "Block mined: " << _shash << std::endl;
}

std::string Block::_calc_hash() const {
  std::stringstream ss;
  ss << _nindex << _sdata << _nnonce << sprev_hash;
  return sha256(ss.str());
}
