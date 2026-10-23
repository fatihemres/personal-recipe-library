import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';
import { SafeMarkdown } from './SafeMarkdown';

it('renders Markdown text without executing HTML, images or unsafe links',()=>{
  const view=render(<SafeMarkdown source={'# Başlık\n**Kalın** <script>alert(1)</script>\n![resim](https://example.com/a.png)\n[kötü](javascript:alert)\n[güvenli](https://example.com)'} />);
  expect(screen.getByRole('heading',{name:'Başlık'})).toBeInTheDocument();
  expect(screen.getByText(/<script>alert\(1\)<\/script>/)).toBeInTheDocument();
  expect(view.container.querySelector('script,img')).toBeNull();
  expect(screen.queryByRole('link',{name:'kötü'})).toBeNull();
  expect(screen.getByRole('link',{name:'güvenli'})).toHaveAttribute('rel','noopener noreferrer');
});
