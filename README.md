## About

Smart contract deployed on the [MultiversX Devnet](https://devnet-explorer.multiversx.com/accounts/erd1qqqqqqqqqqqqqpgqqhrpl2v5mk6wf05006p06phuyksdzu0ltrgqenas7s). It brings chess logic to the blockchain, including move validation, time control and checks.

### Interactions

Has the following endpoints for any user:

- `joinGame()` - any user can sign un for a game by paying a wager
- `makeMove(game_id: usize, from: u8, to: u8)` - make a move inside a game you are playing; `from` and `to` must be between `0` and `63`, representing one of the board's squares (`0` is the equivalent of `a8` and `63` of `h1`)
- `draw(game_id: usize)` - offer a draw for your opponent; if he already offered a draw, the game will end and everyone will get their wager back
- `call_illegal_move(game_id: usize)` - report that your opponent made an illegal move, leaving their king in check; the SC checks the eligibility of the move, then ends the game and sends the caller `wager * 2` if he's right
- `signal_inactivity(game_id: usize)` - signal that your opponent didn't make a move for a full epoch; SC does all the checking, sends `wager * 2` to the caller and ends the game
- `resign(game_id: usize)` - give up on the spot.

The following are the owner-only endpoints:

- `pause` - block all user-only transactions for maintenance
- `unpause` - unblock user-only transactions
- `set_wager(wager_token_id: TokenIdentifier, wager_amount: BigUint)` - sets the wager for all the games; one time only.

And the views are:

- `is_paused()` - returns a `bool`, providing information on the maintenance of the SC
- `wager_token_id()` - returns a `TokenIdentifier`
- `wager_amount()` - returns a `BigUint`
- `score(player: ManagedAddress)` - returns an player's score as `u64`
- `is_player_waiting_opt()` - returns `Option::None` if nobody is waiting for a game; if someone is waiting, returns `Option::Some(ManagedAddress)`
- `games()` - returns all games
- `board(game_id: usize)` - returns the current state of the board for a specific game
- `draw_offer(game_id: usize)` - similar to `is_player_waiting_opt()`, but checks for draw offers within a game.

## Game representation

Each time a game starts, a new `Game` object will come to life.

In order to differentiate through games, each game will have a `game_id`. A game is played by two players (and they are the only ones allowed to make moves), so a `Game` also needs a `white_player` and a `black_player` (we'll also keep track of their kings positions).

At each moment, it is vital to know whose turn is to move, so we'll also use keep track of `player_turn` (which is `0` for white and `1` for black) and of the `last_move_epoch`.

The other things we'll keep track of are the `board` and the `state` of the game.

```
Game = {
	game_id: u64,
	white_player: Address,
	black_player: Address,
	player_turn: u8,
	last_move_epoch: u64,
	board: [u8; 64],
	state: u8
}
```

### Board representation

Initially, I thought of this structure in order to have a clear and well organised, readable code:

```
Board = map(Tile -> Piece)
Tile = (x_position: u8, y_position: u8)
Piece = {
	piece_type: King | Queen | etc...
	color: Black | White
}
```

But as I found space optimisation to be more important, I followed a different approach. Instead of having a chess tile represented as a `(u8, u8)`, it could be just an `u8`. As an example, tile `(5, 6)` can correspond to `5 + 6 * 8 = 53` in this new representation.

Furthermore, piece representation can be simplified into an `u8` as well - an example would be to have the white king as `0`, the black king as `1`, white queen as `2` etc. - so a piece would be just an `u8`.

Alright, so we can, for example, have the king-side white rook in its starting position as `63 -> 4`. But can we store it even more efficient?

The final idea for the board (and the most efficient one I thought of) is to represent it as an `u8` array of length 64 (number of squares on a chess board). Basically, the index at which a piece sits in the array is the piece's tile.

Final piece representation is like so:

```
White: King -> 0, Queen -> 1, Rook -> 2, Bishop -> 3, Knight -> 4, Pawn -> 5
Empty square -> 6
Black: King -> 7, Queen -> 8, Rook -> 9, Bishop -> 10, Knight -> 11, Pawn -> 12
```

So, for example, the initial board layout is `[9, 11, 10, 8, 7, 10, 11, 9, 12, 12, 12, 12, 12, 12, 12, 12, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 2, 4, 3, 1, 0, 3, 4, 2]`.

### Moves representation

A simple way to keep track of moves is to store the piece's position before and after the move. So if I move my knight from `B2` to `C3`, the move will appear as `(57, 42)`. Initially, I wanted to keep track of the moves, but it actually proved to add little to no value to the game for something that takes quite some storage space.

### Game state

The game can have states such as `ongoing`, `white_won`, `black_won` or `stalemate`. They can be easily represented via an `Enum`, but I chose to have just an `u8`. `ongoing` is 0, `white_won` is 1, `black_won` is 2 and `stalemate` is 3.

### Other game storage related things that I considered

##### Not having a king position

For every move, the smart contract requires the mover's king to not enter/remain in check. If there is no specific field for king position, the search takes maximum 64 steps (given the size of the standard chess board). It might not seem like a lot and at first I did not want to have dedicated fields for kings.

##### Board representation as `map(Piece -> Tile)`

It seemed like a pretty cheap way to store the board, but I figured it out that gas fees would be significantly bigger for any move, as verifying a check would take more computational power.

##### No board representation or no moves representation

In the context of a simple game, you do not need to keep track of all the moves to have a correct logic (of course, for a move like en passant to be valid it's needed to know the last move, but still not the entire history of moves). So why did I want to store them? In order for users to be able to come back to their old games and be able to analyse their decisions. It's cheaper to not have a moves history in the storage, but I wanted a better UX.

Alright, but if I have the moves history, the board representation can be easily recreated. Why did I choose to still have a board representation? It's the simple reason that, in order to check the validity of a move inside the smart contract, a lot more computation would be needed. This added computation consists in playing all the moves from the start of the game each time a move is made in order to simulate the current state of board. I did not find this approach to be worth it.

After starting to work on the project, I realised it would be a better idea to keep track of moves off-chain.

##### Another storage style for game state

I mentioned above that an `Enum` could be used as well for the game `state`, but the most elegant approach would be the one bellow.

```
State = {
	Ongoing,
	Over(Outcome)
}

Outcome = {
	White,
	Black,
	Stalemate
}
```

### Types chosen from the MultiversX SC Framework

After a careful read on [MultiversX Docs](https://docs.multiversx.com/developers/developer-reference/storage-mappers/), it was time to choose the most efficient storage mappers for my data.

For the start, `Game` will use the `VecMapper`. I initially thought of using `SingleValueMapper` because it's very cheap, but later on I got to understand that I need a `VecMapper` for this use case.

`Board` is an array which we don't need to iterate through and we know its fixed, short size (64). So a `SingleValueMapper` will do the job.

## Special moves supported

Two-squares pawn advancement is supported, as well as pawn promotion. I did not include castling logic or en passant.
