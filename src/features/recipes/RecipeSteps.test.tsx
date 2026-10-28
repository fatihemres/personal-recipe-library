import { useState } from 'react';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test } from 'vitest';
import { RecipeSteps } from './RecipeSteps';
import { RecipeDetails } from './RecipeDetails';
import type { RecipeInput, Recipe } from '../../shared/contracts/recipe';
const line={id:crypto.randomUUID(),ingredientId:crypto.randomUUID(),quantity:'1.250',unitCode:'g',note:'İnce doğranmış'};
function Harness(){
 const [steps,setSteps]=useState<RecipeInput['steps']>([{id:'first',instructions:'Yağı ısıtın'},{id:'second',instructions:'Karıştırın'}]);
 return <><RecipeSteps steps={steps} onChange={setSteps} ingredients={[line]} names={{[line.ingredientId]:'Şişli mantarı'}}/><output data-testid="steps">{JSON.stringify(steps)}</output></>;
}
test('simple steps expand and retain metadata and line identity across collapse and reorder',async()=>{
 const user=userEvent.setup();render(<Harness/>);
 expect(screen.getByLabelText('1. Talimat')).toHaveValue('Yağı ısıtın');
 const summary=screen.getAllByText('Gelişmiş adım bilgileri (isteğe bağlı)')[0];await user.click(summary);
 const first=summary.closest('details')!;expect(first).toHaveAttribute('open');
 await user.type(within(first).getByLabelText('Adım başlığı'),'Ön hazırlık');
 await user.type(within(first).getByLabelText('Süre (dakika, tam sayı)'),'0');
 await user.click(within(first).getByRole('checkbox'));
 await user.click(summary);expect(first).not.toHaveAttribute('open');
 await user.click(screen.getByRole('button',{name:'1. Adım: Aşağı taşı'}));
 const data=JSON.parse(screen.getByTestId('steps').textContent!);
 expect(data[1]).toMatchObject({id:'first',details:{title:'Ön hazırlık',durationMinutes:'0'},ingredientLineIds:[line.id]});
 await user.click(screen.getAllByText('Gelişmiş adım bilgileri (isteğe bağlı)')[1]);
 expect(screen.getAllByLabelText('Adım başlığı')[1]).toHaveValue('Ön hazırlık');
});
test('unfinished numeric text is retained and marked invalid rather than coerced',async()=>{
 const user=userEvent.setup();render(<Harness/>);await user.click(screen.getAllByText('Gelişmiş adım bilgileri (isteğe bağlı)')[0]);
 await user.type(screen.getAllByLabelText('Süre (dakika, tam sayı)')[0],'1.');
 await user.type(screen.getAllByLabelText('Sıcaklık (tam sayı)')[0],'-');
 expect(screen.getAllByLabelText('Süre (dakika, tam sayı)')[0]).toHaveAttribute('aria-invalid','true');
 expect(JSON.parse(screen.getByTestId('steps').textContent!)[0].details.temperatureValue).toBe('-');
});
test('committed step display preserves explicit temperature units, zero duration and line quantities',()=>{
 const recipe={id:'recipe',kind:'food',servings:'1',prepMinutes:null,cookMinutes:null,ingredients:[line],ingredientNames:{[line.ingredientId]:'Mantar'},notes:null,
 steps:[{id:'step',instructions:'Soteleyin',ingredientLineIds:[line.id],details:{formatVersion:1,title:'Sote',durationMinutes:'0',temperatureValue:'350',temperatureUnit:'F',heatLevel:'Kısık ateş',equipment:null,technique:null,tips:null}}]} as Recipe;
 render(<RecipeDetails recipe={recipe}/>);expect(screen.getByText('350 °F')).toBeInTheDocument();expect(screen.getAllByText(/1.250 g/).length).toBe(2);expect(screen.getByText('0')).toBeInTheDocument();
});
