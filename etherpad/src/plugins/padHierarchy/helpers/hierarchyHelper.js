// class
function HierarchyParser(){
	// this basically goes down and then checks back up the chain for the closest grouping
	this.parse = function(titles){
		var chain = [];
		var result = { id:"", children:[] };
		for(var i = 0; i < titles.length; i++){
			var obj = {children:[]};
			obj.id = titles[i];
			for(var j = chain.length-1; j >= 0; j --){
				var link = chain[j];
				if( obj.id.match("^" + link.id)){
					link.children.push(obj);
					obj.parent = link;
					break;
				}
			}
			if(!obj.parent){
				result.children.push(obj);
			}
			chain.push(obj);
		}
		return result;	
	}
}

// helper method
function getHierarchy(titles){
	return new HierarchyParser().parse(titles);
}
