/** Intervali ispitivanja dok AI ili obračun rade (ADR-022 t. 8). Testovi ih skraćuju. */
export const poll = {
  conversationMs: 1500,
  verdictMs: 2000,
  roundMs: 2000,
  feedMs: 2000,
  slowRoundMs: 150_000,
  maxFeedPolls: 60,
};
