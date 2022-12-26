require("../../steps/init");
const given = require("../../steps/given");
const when = require("../../steps/when");
const chance = require("chance").Chance();

const getRestaurant = () => {
  const restaurant = {
    id: chance.guid(),
    name: chance.string({ length: 16 })
  };

  return restaurant;
};

describe("Given at least one restaurant is in the database", () => {
  beforeAll(async () => {
    await given.restaurant_exists_in_dynamodb(getRestaurant());
  });

  describe("Given an authenticated user", () => {
    let user;
    beforeAll(async () => {
      user = await given.an_authenticated_user();
    });

    it("GET /restaurants should return at least one restaurant", async () => {
      const result = await when.we_invoke_list_restaurants_remotely(user, 1);

      expect(result.statusCode).toEqual(200);
      expect(result.body.restaurants).toHaveLength(1);
    });

    describe("Given at least two restaurants in the database", () => {
      beforeAll(async () => {
        await Promise.all([
          await given.restaurant_exists_in_dynamodb(getRestaurant()),
          await given.restaurant_exists_in_dynamodb(getRestaurant())
        ]);
      });

      it("GET /restaurants should allow for pagination", async () => {
        const firstResult = await when.we_invoke_list_restaurants_remotely(
          user,
          1
        );
        expect(firstResult.statusCode).toEqual(200);

        const { nextToken } = firstResult.body;
        expect(nextToken).toBeDefined();

        const secondResult = await when.we_invoke_list_restaurants_remotely(
          user,
          1,
          nextToken
        );

        expect(secondResult.statusCode).toEqual(200);
        expect(secondResult.body.restaurants).toHaveLength(1);
        expect(firstResult.body.restaurants).not.toEqual(
          secondResult.body.restaurants
        );
      });
    });
  });
});
