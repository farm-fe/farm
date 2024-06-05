import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { UsersController } from './users.controller';
import { AController } from './a.controller';
import { BController } from './b.controller';

@Module({
  imports: [],
  controllers: [AppController, UsersController, AController, BController],
  providers: [AppService],
})
export class AppModule {}
