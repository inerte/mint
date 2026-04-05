import React, { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import * as sigilRaw from './generated/minesweep-domain';

type Cell = { adjacent: number; bomb: boolean; flagged: boolean; revealed: boolean; x: number; y: number };
type Game = { board: Cell[]; height: number; status: string; width: number };
type Mode = 'reveal' | 'flag';
type Point = { x: number; y: number };

const BOARD_HEIGHT = 6;
const BOARD_WIDTH = 6;
const TOTAL_BOMBS = 6;

type SigilDomain = {
  gameFromBombs: (points: Point[]) => Promise<Game>;
  revealCell: (game: Game, targetX: number, targetY: number) => Promise<Game>;
  toggleFlag: (game: Game, targetX: number, targetY: number) => Promise<Game>;
};

const sigil = sigilRaw as unknown as SigilDomain;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function boardRows(game: Game): Cell[][] {
  const rows: Cell[][] = [];
  for (let y = 0; y < game.height; y += 1) {
    rows.push(game.board.filter((cell) => cell.y === y).sort((a, b) => a.x - b.x));
  }
  return rows;
}

function cellLabel(cell: Cell, status: string): string {
  if (cell.revealed) {
    if (cell.bomb) return '✹';
    return cell.adjacent === 0 ? '' : String(cell.adjacent);
  }
  if (status === 'lost' && cell.bomb) return '✹';
  if (cell.flagged) return '⚑';
  return '';
}

function statusText(game: Game): string {
  const hiddenSafe = game.board.filter((cell) => !cell.bomb && !cell.revealed).length;
  switch (game.status) {
    case 'lost':
      return 'Boom. Restart loads a fresh randomized board.';
    case 'won':
      return 'Board cleared. Zero-adjacent cells cascade open automatically.';
    default:
      return `${hiddenSafe} safe cells left`;
  }
}

function randomBombPositions(): Point[] {
  const cells: Point[] = [];
  for (let y = 0; y < BOARD_HEIGHT; y += 1) {
    for (let x = 0; x < BOARD_WIDTH; x += 1) {
      cells.push({ x, y });
    }
  }
  for (let index = cells.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    const current = cells[index];
    cells[index] = cells[swapIndex];
    cells[swapIndex] = current;
  }
  return cells.slice(0, TOTAL_BOMBS);
}

function MinesweepApp(): JSX.Element {
  const [game, setGame] = useState<Game | null>(null);
  const [mode, setMode] = useState<Mode>('reveal');
  const [appError, setAppError] = useState<string | null>(null);
  const gameRef = useRef<Game | null>(null);

  useEffect(() => {
    gameRef.current = game;
  }, [game]);

  useEffect(() => {
    void sigil.gameFromBombs(randomBombPositions())
      .then((nextGame) => {
        setGame(nextGame);
        setAppError(null);
      })
      .catch((error) => setAppError(errorMessage(error)));
  }, []);

  async function restart(): Promise<void> {
    try {
      const nextGame = await sigil.gameFromBombs(randomBombPositions());
      setGame(nextGame);
      setMode('reveal');
      setAppError(null);
    } catch (error) {
      setAppError(errorMessage(error));
    }
  }

  async function applyMove(targetX: number, targetY: number, nextMode: Mode): Promise<void> {
    const current = gameRef.current;
    if (!current) return;
    try {
      const nextGame = nextMode === 'flag'
        ? await sigil.toggleFlag(current, targetX, targetY)
        : await sigil.revealCell(current, targetX, targetY);
      setGame(nextGame);
      setAppError(null);
    } catch (error) {
      setAppError(errorMessage(error));
    }
  }

  if (!game) {
    return (
      <div className="minesweep-shell">
        <p className="minesweep-banner">Loading Minesweep bridge...</p>
      </div>
    );
  }

  const flaggedCount = game.board.filter((cell) => cell.flagged).length;
  const canInteract = game.status === 'playing';

  return (
    <div className="minesweep-shell">
      <header className="minesweep-header">
        <div>
          <p className="eyebrow">Projects / Sigil Minesweep</p>
          <h1>Sigil Minesweep</h1>
          <p className="subtitle">A browser demo where Sigil owns randomized board state and the React bridge owns the interaction layer.</p>
        </div>
        <button className="restart" onClick={() => void restart()}>Restart</button>
      </header>
      <section className="minesweep-toolbar">
        <div className="status-group">
          <span className="status-pill">{statusText(game)}</span>
          <span className="status-pill muted">{flaggedCount} flags placed</span>
        </div>
        <div className="mode-group" role="tablist" aria-label="Cell action">
          <button type="button" data-active={mode === 'reveal'} onClick={() => setMode('reveal')} role="tab" aria-selected={mode === 'reveal'}>
            Reveal
          </button>
          <button type="button" data-active={mode === 'flag'} onClick={() => setMode('flag')} role="tab" aria-selected={mode === 'flag'}>
            Flag
          </button>
        </div>
      </section>
      {appError ? <p className="minesweep-banner error">App error: {appError}</p> : null}
      <section className="minesweep-board" aria-label="Minesweep board">
        {boardRows(game).map((row, index) => (
          <div className="minesweep-row" key={index} style={{ gridTemplateColumns: `repeat(${game.width}, minmax(0, 1fr))` }}>
            {row.map((cell) => {
              const tone = cell.revealed
                ? cell.bomb
                  ? 'bomb'
                  : cell.adjacent === 0
                    ? 'cleared'
                    : `adj-${cell.adjacent}`
                : cell.flagged
                  ? 'flagged'
                  : 'hidden';
              return (
                <button
                  key={`${cell.x}-${cell.y}`}
                  type="button"
                  className="cell"
                  data-tone={tone}
                  disabled={!canInteract && !cell.flagged && !cell.revealed}
                  onClick={() => { if (canInteract) void applyMove(cell.x, cell.y, mode); }}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    if (canInteract) void applyMove(cell.x, cell.y, 'flag');
                  }}
                  aria-label={`Cell ${cell.x + 1}, ${cell.y + 1}`}
                >
                  {cellLabel(cell, game.status)}
                </button>
              );
            })}
          </div>
        ))}
      </section>
      <p className="minesweep-banner muted">
        Left click uses the selected mode. Right click always toggles a flag. Reveal a zero-adjacent cell and the surrounding empty region opens automatically, like classic Minesweeper.
      </p>
    </div>
  );
}

export function mountMinesweepApp(element: HTMLElement): void {
  createRoot(element).render(<MinesweepApp />);
}
