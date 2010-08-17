import('etherpad.log');
// class
function HierarchyParser(){
	
	// this basically goes down and then checks back up the chain for the closest grouping
	this.parse = function(titles, topLevelLabel){
		var chain = [];
		var result = { id:topLevelLabel, shortName:topLevelLabel, children:[] };
		for(var i = 0; i < titles.length; i++){
			var obj = {children:[]};
			obj.id =titles[i];
			
			for(var j = chain.length-1; j >= 0; j --){
				var link = chain[j];
				if( obj.id.match("^" + link.id)){
					link.children.push(obj);
					obj.parent = link;
					break;
				}
			}
			if(!obj.parent){
				obj.parent = result;
				result.children.push(obj);
			}
			obj.shortName = obj.id.replace(new RegExp("^" + obj.parent.id + "-"), '');
			obj.path = getPath(obj);
			chain.push(obj);
		}
		return result;	
	}
	function getPath(obj){
		var parent = obj;
		var result = [];
		while(parent){
			result.unshift(parent.shortName);
			parent = parent.parent;
		}
		return '/specs/' + result.join('/');
	}
}

// helper method
function getHierarchy(titles, topLevelLabel){
	return new HierarchyParser().parse(titles,topLevelLabel);
}
