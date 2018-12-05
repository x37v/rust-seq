* if the trigger node had a list of value nodes then we could support multiple triggers at a time
	* so we can use a single trigger for midi notes and supply note value, velocity, etc with it
	* we would remove the ability to schedule a value directly, it would have to be done in a trigger
		* so an trigger in the graph would have to hold all the appropriate parameter bindings and store them
		  when triggering [midi note, osc, etc etc]
  * maybe, to make things easy, CC and other parameters would be a different trigger index


* can we make a macro to reduce the boilerplate for children_max and remove if
  has no children?
* can the macro that specifies the serde parameters also register the serde
  parameters to a factory for creating these objects?


# serialize format? 

bindings:
    - id: <uuid>
      alias: <optionalName>
      type: <typename>
      params: #for instance, a cast would have an input format, dest format, binding to cast
        - name: value
        - name: value
        - name: value

nodes: #both graph and non graph nodes??
    - id: <uuid>
      type: <typename>
      alias: <optionalName>
      params:
        - name: value
        - name: value
        - name: value
      children:
        - <uuid>
        - <uuid>
      meta: #optional
        - location: (x, y)

triggers:
    - id: <uuid>
      type: <typename>
      alias: <optionalName>
      params: #aka bindings
        - name: value
        - name: value
