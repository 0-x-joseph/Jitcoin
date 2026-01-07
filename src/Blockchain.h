/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Blockchain.h                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ybouryal <ybouryal@student.1337.ma>        +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2026/01/07 02:25:33 by ybouryal          #+#    #+#             */
/*   Updated: 2026/01/07 03:27:06 by ybouryal         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#ifndef __BLOCKCHAIN__
#define __BLOCKCHAIN__

#include "Block.h"
#include <cstdint>
#include <vector>

class Blockchain {
private:
  uint32_t _ndifficulty;
  std::vector<Block> _vchain;

  Block _get_last_block() const;

public:
  Blockchain();

  ~Blockchain();

  void add_block(Block b_new);
};

#endif /* __BLOCKCHAIN__ */
