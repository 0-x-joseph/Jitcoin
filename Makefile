# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Makefile                                           :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: ybouryal <ybouryal@student.1337.ma>        +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2026/01/07 04:26:57 by ybouryal          #+#    #+#              #
#    Updated: 2026/01/07 04:27:58 by ybouryal         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

CC = c++
CFLAGS = 
LDFLAGS =

BUILD_DIR = build
SRC_DIR = src

SRC_FILES= $(SRC_DIR)/Block.cpp \
					 $(SRC_DIR)/Blockchain.cpp \
					 $(SRC_DIR)/sha256.cpp \
					 $(SRC_DIR)/main.cpp

EXE=$(BUILD_DIR)/jitcoin

all:$(EXE)

$(EXE): $(BUILD_DIR)
	@$(CC) $(CFLAGS) $(LDFLAGS) $(SRC_FILES) -o $@


$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

clean:
	@rm -rf $(BUILD_DIR)

.PHONY: clean all $(EXE) $(BUILD_DIR)
