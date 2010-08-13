// class
function HierarchyParser(){
	this.parse = function(titles){
		
		var result = getUniquePrefixes(titles,0);
		
		return result;	
	}
	function getUniquePrefixes(titles,startIndex){
		var result = [];
		for(var i=startIndex; i< titles.length;i++){
			var match = getChildren(titles[i], titles, i + 1)
			i = match[0]; // skip to nextIndex
			if(match[1].length>0) result.push(match[1]);
		}
		return result;
	}
	// return [nextIndex,matches]
	function getChildren(parent, titles, startIndex){
		var result = [];
		var nextIndex = startIndex;
		jstestdriver.console.log("starting from " + startIndex + ", trying to match " + parent);
		for(var i = startIndex; i < titles.length; i++){
			var title = titles[i];
			if( title.match( "^" + parent) ){
				var children = getUniquePrefixes(titles,i+1);
				jstestdriver.console.log("matched " + parent + " so adding " + title + " with " + children.length + " children");
				if (children.length > 0) {
					var object = {};
					object[title] = children;
					result.push(object);
					i += children.length; // not quite right but might work..
				}else{
					result.push(title);
					nextIndex = i;
				}
				
			}
		}
		
		return [nextIndex, result]; 
	}
	function getNextTopLevel(titles, excludes){
		var result = null;
		var maxMatchedChars;
		for(var title in titles){
			if(title.match('/^' + maxMatchedChars + '/')) maxMatchedChars = title;
		}
		return result;
	}
}

// helper method
function getHierarchy(titles){
	return new HierarchyParser().parse(titles);
}
