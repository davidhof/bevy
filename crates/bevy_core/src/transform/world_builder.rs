use bevy_app::ComponentSet;
use bevy_transform::components::{LocalTransform, Parent};
use legion::{
    filter::{ChunksetFilterData, Filter},
    prelude::*,
    world::{IntoComponentSource, TagLayout, TagSet},
};

pub trait WorldBuilderSource {
    fn build(&mut self) -> WorldBuilder;
}

impl WorldBuilderSource for World {
    fn build(&mut self) -> WorldBuilder {
        WorldBuilder {
            world: self,
            current_entity: None,
            parent_entity: None,
        }
    }
}

pub struct WorldBuilder<'a> {
    world: &'a mut World,
    current_entity: Option<Entity>,
    parent_entity: Option<Entity>,
}

impl<'a> WorldBuilder<'a> {
    pub fn entity(&mut self) -> &mut Self {
        let entity = *self.world.insert((), vec![()]).first().unwrap();
        self.current_entity = Some(entity);
        self.add_parent_to_current_entity();
        self
    }

    pub fn set_entity(&mut self, entity: Entity) -> &mut Self {
        self.current_entity = Some(entity);
        self
    }

    /// note: this is slow and does a full entity copy
    pub fn with<T>(&mut self, component: T) -> &mut Self
    where
        T: legion::storage::Component,
    {
        let _ = self
            .world
            .add_component(*self.current_entity.as_ref().unwrap(), component);
        self
    }

    pub fn entities<C>(&mut self, components: C) -> &mut Self
    where
        C: IntoComponentSource,
    {
        self.world.insert((), components);
        self
    }

    pub fn entity_with(&mut self, component_set: impl ComponentSet) -> &mut Self {
        let current_entity = component_set.insert(self.world);
        self.current_entity = Some(current_entity);
        self.add_parent_to_current_entity();
        self
    }

    pub fn with_children(&mut self, mut build_children: impl FnMut(&mut Self) -> &mut Self) -> &mut Self {
        self.parent_entity = self.current_entity;
        self.current_entity = None;

        build_children(self);

        self.current_entity = self.parent_entity;
        self.parent_entity = None;
        self
    }

    fn add_parent_to_current_entity(&mut self) {
        let current_entity = self.current_entity.unwrap();
        if let Some(parent_entity) = self.parent_entity {
            let _ = self
                .world
                .add_component(current_entity, Parent(parent_entity));
            let _ = self
                .world
                .add_component(current_entity, LocalTransform::identity());
        }
    }
}

pub trait CommandBufferBuilderSource {
    fn build(&mut self) -> CommandBufferBuilder;
}

impl CommandBufferBuilderSource for CommandBuffer {
    fn build(&mut self) -> CommandBufferBuilder {
        CommandBufferBuilder {
            command_buffer: self,
            current_entity: None,
            parent_entity: None,
            parent_entities: Vec::new(),
        }
    }
}

pub struct CommandBufferBuilder<'a> {
    command_buffer: &'a mut CommandBuffer,
    current_entity: Option<Entity>,
    parent_entities: Vec<Entity>,
    parent_entity: Option<Entity>,
}

impl<'a> CommandBufferBuilder<'a> {
    pub fn entity(&mut self) -> &mut Self {
        let entity = *self.command_buffer.insert((), vec![()]).first().unwrap();
        self.current_entity = Some(entity);
        self.add_parent_to_current_entity();
        self
    }

    pub fn set_entity(&mut self, entity: Entity) -> &mut Self {
        self.current_entity = Some(entity);
        self
    }

    // note: this is slow and does a full entity copy
    pub fn with<T>(&mut self, component: T) -> &mut Self
    where
        T: legion::storage::Component,
    {
        let _ = self
            .command_buffer
            .add_component(*self.current_entity.as_ref().unwrap(), component);
        self
    }

    pub fn entities<T, C>(&mut self, tags: T, components: C) -> &mut Self
    where
        T: TagSet + TagLayout + for<'b> Filter<ChunksetFilterData<'b>> + 'static,
        C: IntoComponentSource + 'static,
    {
        self.command_buffer.insert(tags, components);
        self
    }

    pub fn entity_with(&mut self, component_set: impl ComponentSet) -> &mut Self {
        let current_entity = component_set.insert_command_buffer(self.command_buffer);
        self.current_entity = Some(current_entity);
        self.add_parent_to_current_entity();
        self
    }

    pub fn with_children(&mut self, mut build_children: impl FnMut(&mut Self) -> &mut Self) -> &mut Self {
        let current_entity = self.current_entity.expect("Cannot add children without a parent. Try creating an entity first.");
        self.parent_entities.push(current_entity);
        self.parent_entity = Some(current_entity);
        self.current_entity = None;

        build_children(self);

        self.current_entity = self.parent_entities.pop();
        self.parent_entity = if self.parent_entities.is_empty() { None } else { self.parent_entities.last().cloned() };
        self
    }

    fn add_parent_to_current_entity(&mut self) {
        let current_entity = self.current_entity.unwrap();
        if let Some(parent_entity) = self.parent_entity {
            let _ = self
                .command_buffer
                .add_component(current_entity, Parent(parent_entity));
            let _ = self
                .command_buffer
                .add_component(current_entity, LocalTransform::identity());
        }
    }
}