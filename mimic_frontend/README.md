# Requirements
## custom shader
need to be able to create a custom shader in a graphics backend agnostic way and have 
a mesh render with this custom shader
the shader needs
- uniform data -> layout binding
- texture sampler -> layout binding
- inputs (positions, texture coords and more) -> layout location
- outputs -> layout location?


render_graph -> consists of multiple connected graphs which are DAGs
each render pass is a connected DAG
the nodes in a DAG represent an execution of command buffers (?) (or a compute? is that)
they need to be associated with a graphics pipeline to be able to bind the pipeline from the command buffers

we need it to automagically create graphics pipelines for us with render pass and subpasses
what we need is some way to have graphics pipelines (blueprints for graphics setup) prepared to be instanced 
then we need some way to send command buffer commands
then we need it to hide the synchronization from us. In vulkan and prolly d12 we need to make sure it happens
but in opengl and d11 the driver does the synchronization for us


- command buffer -> set of commands to send to a queue for execution
(primary command buffer can be made up of secondary command buffers to break it up into things that don't change everyframe)

- render pass -> encapsulates work done on the framebuffer attachments -> this means clearing, getting them ready for shaders to write into them etc

- more complex buffer setups can be achieved by breaking the render pass into multiple subpasses
these subpasses each define states that buffers/attachments need to have on input and output
HOWEVER: not every shader can be done as a subpass. This is because subpasses has a restriction that a pixel in a subpass can
only depend on the value of the same pixel in a previous subpass. So not guaranteed that region around the pixel will be in the right state