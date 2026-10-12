import type { LibraryPage } from '../../shared/contracts/ingredientLibrary';

/** Preserve the database hierarchy in both browsing and personal assignment. */
export function orderedCategoryOptions(categories: LibraryPage['categories']) {
  const result: { id: string; label: string; count: number }[] = [];
  const visited = new Set<string>();
  function branch(parent: string | null, depth: number) {
    for (const {category,count} of categories.filter(c => c.category.parentId === parent)
      .sort((a,b) => a.category.tr.localeCompare(b.category.tr,'tr'))) {
      if (visited.has(category.id)) continue;
      visited.add(category.id);
      result.push({id:category.id,label:`${'— '.repeat(depth)}${category.tr}`,count});
      branch(category.id,depth+1);
    }
  }
  branch(null,0);
  return result;
}
