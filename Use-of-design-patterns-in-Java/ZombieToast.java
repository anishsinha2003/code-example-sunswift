package dungeonmania.entities.enemies;

import java.util.Random;

import dungeonmania.Game;
import dungeonmania.util.Position;

public class ZombieToast extends Enemy {
    public static final double DEFAULT_HEALTH = 5.0;
    public static final double DEFAULT_ATTACK = 6.0;
    private Random randGen = new Random();

    private MovementStrategy moveStrategy = new ZombieToastMovementStrategy();

    public ZombieToast(Position position, double health, double attack) {
        super(position, health, attack);
    }

    // implemented the strategy design here
    @Override
    public void move(Game game) {
        moveStrategy.move(game, this);
    }

    public Random getRandomGen() {
        return this.randGen;
    }


}
