const searchRoot=document.querySelector("[data-search-root]");

if(searchRoot){
  const dropdown=searchRoot.querySelector("[data-search-dropdown]");
  const emptyMessage=searchRoot.querySelector("[data-search-empty]");
  const input=searchRoot.querySelector("[data-search-input]");
  const results=searchRoot.querySelector("[data-search-results]");
  const status=searchRoot.querySelector("[data-search-status]");
  const indexUrl=searchRoot.getAttribute("data-search-index-url");

  const normalize=(text)=>text.toLowerCase().trim();

  const countOccurrences=(haystack,needle)=>{
    if(needle.length===0){
      return 0;
    }

    let count=0;
    let index=0;

    while(index<haystack.length){
      const found=haystack.indexOf(needle,index);

      if(found<0){
        return count;
      }

      count+=1;
      index=found+needle.length;
    }

    return count;
  };

  const queryTerms=(query)=>query.split(/\s+/).filter((term)=>term.length>0);

  const scoreEntry=(entry,query,index)=>{
    const title=normalize(entry.title);
    const description=normalize(entry.description);
    const section=normalize(entry.section);
    const text=normalize(entry.text);
    const terms=queryTerms(query);
    const exactTitle=title===query;
    const titleStarts=title.startsWith(query);
    const titleIncludes=title.includes(query);
    const descriptionIncludes=description.includes(query);
    const sectionExact=section===query;
    const textIncludes=text.includes(query);
    const termScore=terms.reduce((score,term)=>{
      return score
        +(countOccurrences(title,term)*40)
        +(countOccurrences(description,term)*12)
        +(countOccurrences(section,term)*8)
        +(Math.min(countOccurrences(text,term),6)*4);
    },0);

    if(!(titleIncludes||descriptionIncludes||sectionExact||textIncludes)){
      return null;
    }

    return {
      entry,
      index,
      score:
        (exactTitle?1000:0)
        +(titleStarts?700:0)
        +(titleIncludes?350:0)
        +(descriptionIncludes?120:0)
        +(sectionExact?80:0)
        +(textIncludes?25:0)
        +termScore
    };
  };

  const renderResult=(entry,query)=>{
    const item=document.createElement("li");
    item.className="search-result";

    const link=document.createElement("a");
    link.className="search-result-link";
    link.href=entry.url;
    link.textContent=entry.title;

    const meta=document.createElement("div");
    meta.className="search-result-meta";
    meta.textContent=`${entry.section} · ${entry.url}`;

    const description=document.createElement("div");
    description.className="search-result-desc";
    description.textContent=entry.description.length>0?entry.description:entry.text.slice(0,180);

    item.appendChild(link);
    item.appendChild(meta);
    item.appendChild(description);

    if(query.length>0){
      item.dataset.query=query;
    }

    return item;
  };

  const setStatus=(message)=>{status.textContent=message;};

  const showDropdown=(visible)=>{dropdown.hidden=!visible;};

  const showEmpty=(visible)=>{emptyMessage.hidden=!visible;};

  const clearResults=()=>{results.replaceChildren();};

  const searchEntries=(entries,query)=>{
    if(query.length===0){
      return [];
    }

    return entries.map((entry,index)=>scoreEntry(entry,query,index))
      .filter((entry)=>entry!==null)
      .sort((left,right)=>{
        if(left.score===right.score){
          return left.index-right.index;
        }

        return right.score-left.score;
      })
      .slice(0,20)
      .map((entry)=>entry.entry);
  };

  fetch(indexUrl).then((response)=>{
    if(!response.ok){
      throw new Error(`search index request failed: ${response.status}`);
    }

    return response.json();
  }).then((entries)=>{
    setStatus(`Loaded ${entries.length} indexed pages.`);

    input.addEventListener("input",()=>{
      const query=normalize(input.value);
      const matches=searchEntries(entries,query);

      clearResults();

      if(query.length===0){
        showDropdown(false);
        showEmpty(false);
        return;
      }

      showDropdown(true);
      showEmpty(matches.length===0);
      setStatus(matches.length===0?`No results for "${query}".`:`${matches.length} result${matches.length===1?"":"s"} for "${query}".`);
      matches.forEach((entry)=>results.appendChild(renderResult(entry,query)));
    });

    input.addEventListener("focus",()=>{
      if(normalize(input.value).length>0){
        showDropdown(true);
      }
    });

    document.addEventListener("click",(event)=>{
      if(!searchRoot.contains(event.target)){
        showDropdown(false);
      }
    });
  }).catch((error)=>{
    clearResults();
    showDropdown(true);
    showEmpty(true);
    setStatus(`Search is unavailable: ${error.message}`);
  });
}
