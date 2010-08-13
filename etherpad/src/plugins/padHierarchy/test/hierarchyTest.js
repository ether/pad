HierarchyTest = TestCase("HierarchyTest");

HierarchyTest.prototype.testNestingDetection = function(){
	var titles = [
		"maverick", 
		"maverick-installer",
		"maverick-installer-partitioner",
		"maverick-installer-partitioner-when-other-os-installed",
		"maverick-installer-setup-process",
		"maverick-installer-setup-process-get-ready",
		"maverick-installer-setup-process-keyboard-layout",
		"maverick-installer-setup-process-slideshow",
		"maverick-installer-setup-process-welcome",
		"maverick-installer-setup-process-wifi-prompt",
		"maverick-installer-user-account-create",
		"maverick-software-center",
		"maverick-software-center-apturl",
		"maverick-software-center-buy",
		"maverick-software-center-buying",
		"maverick-software-center-deauthorizing",
		"maverick-software-center-department",
		"maverick-software-center-department-subsection",
		"maverick-software-center-lobby",
		"maverick-software-center-reinstalling-previous-purchases",
		"maverick-software-center-whats-new",
		"people",
		"people-michaelforrest",
		"people-michaelforrest-ubuntu-spec-editor",
		"people-michaelforrest-ubuntu-spec-editor-authentication"
	];
	
	var expectation = [
		{
			id:"maverick", children: [
				{id:"software-center", children: [
					{id:"lobby", children:[]}, 
					{id:"apturl", children:[]},
					{id:"buy", children:[]} , 
					{id:"buying",children:[]},
					{id:"deauthorizing", children:[]},
					{id:"department", children: [ {id:"subsection", children:[]}]},
					{id:"reinstalling-previous-purchases",children:[]},
					{id:"whats-new",children:[]}]
			}, 
				{id:"installer",children:[
					{id:"partitioner", children: [{
						id: "when-other-os-installed",
						children: []
					}]
				}, {id:"setup-process", children: [
					{
						id: "get-ready",
						children: []
					},
					{
						id: "keyboard-layout",
						children: []
					}, {
						id: "slideshow",
						children: []
					}, {
						id: "welcome",
						children: []
					},{
						id: "wifi-prompt",
						children: []
					}]
				}, {
					id: "user-account-create",
					children: []
				}]
			}]
		},
		{
			id:"people", children: [
				{id:"michaelforrest", children: [
					{id:"ubuntu-spec-editor",children: [ {id:"authentication", children:[] }]}
				]}
			]
		}
	];
	var result = getHierarchy(titles);
	assertEquals("should be two children on the root (maverick and people) - result was " + result.children, 2, result.children.length);
	assertEquals("should be two children under maverick (installer and software center)",2, result.children[0].children.length );
	//assertEquals("maverick", result[0][0]);
	//assertEquals(expectation,getHierarchy(titles), "should have right number of top-level structures");
	
};

  
  