/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Block.h                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ybouryal <ybouryal@student.1337.ma>        +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2026/01/07 02:19:25 by ybouryal          #+#    #+#             */
/*   Updated: 2026/01/07 03:17:43 by ybouryal         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#ifndef __BLOCK__
#define __BLOCK__

#include <cstdint>
#include <ctime>
#include <openssl/sha.h>
#include <string>

class Block {
private:
  uint32_t _nindex;
  uint64_t _nnonce;
  std::string _sdata;
  std::string _shash;
  time_t *_ttime;

  std::string _calc_hash() const;

public:
  std::string sprev_hash;

  Block(uint32_t nindexin, const std::string &sdatain);

  ~Block();

  std::string get_hash();

  void mine_block(uint32_t ndifficulty);
};

#endif /* __BLOCK__ */
