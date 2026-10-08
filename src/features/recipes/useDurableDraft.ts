import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import type { Draft, RecipeInput } from '../../shared/contracts/recipe';
import { registerDraftExitGuard } from '../../app/nativeExit';
import { DraftSession } from './DraftSession';
export function useDurableDraft(
  client: RecipeClient,
  input: RecipeInput,
  dirty: boolean,
  recovered?: Draft,
) {
  const [session] = useState(() => new DraftSession(client, recovered));
  const latest = useRef({ input, dirty });
  const [status, setStatus] = useState<'idle' | 'saving' | 'saved' | 'failed'>(
    recovered ? 'saved' : 'idle',
  );
  const [failure, setFailure] = useState<unknown>();
  const [closing, setClosing] = useState(false);
  useLayoutEffect(() => {
    latest.current = { input, dirty };
  }, [input, dirty]);
  async function flush() {
    if (!latest.current.dirty) return;
    setStatus('saving');
    try {
      await session.flush(latest.current.input);
      setStatus('saved');
      setFailure(undefined);
    } catch (e) {
      setStatus('failed');
      setFailure(e);
      throw e;
    }
  }
  useEffect(() => {
    if (!dirty) return;
    const persist = () => {
      setStatus('saving');
      void session.flush(input).then(
        () => {
          setStatus('saved');
          setFailure(undefined);
        },
        (e) => {
          setStatus('failed');
          setFailure(e);
        },
      );
    };
    const timer = setTimeout(persist, 500);
    return () => clearTimeout(timer);
  }, [input, dirty, session]);
  useEffect(() => {
    const timer = setInterval(() => {
      const value = latest.current;
      if (value.dirty) {
        void session.flush(value.input).then(
          () => {
            setStatus('saved');
            setFailure(undefined);
          },
          (e) => {
            setStatus('failed');
            setFailure(e);
          },
        );
      }
    }, 2000);
    return () => clearInterval(timer);
  }, [session]);
  useEffect(
    () =>
      registerDraftExitGuard(async () => {
        setClosing(true);
        try {
          const value = latest.current;
          if (value.dirty) await session.flush(value.input);
        } catch (e) {
          setClosing(false);
          setStatus('failed');
          setFailure(e);
          throw e;
        }
      }),
    [session],
  );
  const displayStatus =
    dirty &&
    status !== 'failed' &&
    status !== 'saving' &&
    !session.matches(input)
      ? 'pending'
      : status;
  return { session, status: displayStatus, failure, flush, closing };
}
