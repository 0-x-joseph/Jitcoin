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
CFLAGS = -Werror -Wextra -Wall
LDFLAGS =
PWD = $(pwd)

BUILD_DIR = $(PWD)/build
SRC_DIR = $(PWD)/src

SRC_FILES= $(SRC_DIR)/Block.cpp \
					 $(SRC_DIR)/Blockchain.cpp \
					 $(SRC_DIR)/sha256.cpp \
					 $(SRC_DIR)/main.cpp

EXE=jitcoin

# create the build directory
# cd $BUILD_DIR
all:
	echo $(PWD)
	echo $(LDFLAGS)

all:$(EXE)

$(EXE): $(BUILD_DIR)
	@$(CC) $(CFLAGS) $(LDFLAGS) $(SRC_FILES) -o $@


$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

clean:
	@rm -rf $(BUILD_DIR)

.PHONY: clean all $(EXE) $(BUILD_DIR)
