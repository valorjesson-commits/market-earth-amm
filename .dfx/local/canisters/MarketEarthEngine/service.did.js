export const idlFactory = ({ IDL }) => {
  const Result = IDL.Variant({ 'ok' : IDL.Nat, 'err' : IDL.Text });
  return IDL.Service({
    'executeDualSwap' : IDL.Func([IDL.Nat, IDL.Bool], [Result], []),
    'getReserves' : IDL.Func([], [IDL.Nat, IDL.Nat], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
