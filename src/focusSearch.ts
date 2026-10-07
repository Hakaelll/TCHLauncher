export function focusSearch(root:HTMLElement|null) {
  const search=Array.from(root?.querySelectorAll<HTMLInputElement>('input[type="search"]')??[]).find(input=>
    !input.matches(':disabled')&&!input.closest('[hidden],[inert],[aria-hidden="true"]')&&input.getClientRects().length>0)
  if(!search)return false
  search.focus();search.select();return true
}
