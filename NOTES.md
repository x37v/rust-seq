* if the trigger node had a list of value nodes then we could support multiple triggers at a time
	* so we can use a single trigger for midi notes and supply note value, velocity, etc with it
	* we would remove the ability to schedule a value directly, it would have to be done in a trigger
		* so an trigger in the graph would have to hold all the appropriate parameter bindings and store them
		  when triggering [midi note, osc, etc etc]
  * maybe, to make things easy, CC and other parameters would be a different trigger index
